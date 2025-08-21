use serde::Serialize;
use std::io::{self, Read};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use ureq;

// ----------------- Hardcoded configuration -----------------
const WORKER_THREADS: usize = 50;              
const REQUEST_TIMEOUT_SECS: u64 = 5;           
const MAX_RETRIES: usize = 2;                 
const PERIODIC_INTERVAL_SECS: Option<u64> = None;

// ----------------- Types -----------------
#[derive(Debug, Serialize, Clone)]
struct WebsiteStatus {
    url: String,
    status: Result<u16, String>,
    response_time_ms: u128,
    timestamp_epoch_secs: u64,
}

#[derive(Clone)]
struct Config {
    timeout: Duration,
    max_retries: usize,
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

fn build_agent(timeout: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build()
}


fn check_site(url: &str, cfg: &Config) -> WebsiteStatus {
    let agent = build_agent(cfg.timeout);
    let start = Instant::now();
    let mut last_err: Option<String> = None;

    for attempt in 0..=cfg.max_retries {
        match agent.get(url).call() {
            Ok(resp) => {
                return WebsiteStatus {
                    url: url.to_string(),
                    status: Ok(resp.status() as u16),
                    response_time_ms: start.elapsed().as_millis(),
                    timestamp_epoch_secs: now_epoch_secs(),
                };
            }
            Err(e) => {
                last_err = Some(format!("{}", e));
                if attempt < cfg.max_retries {
                    thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                }
            }
        }
    }

    WebsiteStatus {
        url: url.to_string(),
        status: Err(last_err.unwrap_or_else(|| "unknown error".to_string())),
        response_time_ms: start.elapsed().as_millis(),
        timestamp_epoch_secs: now_epoch_secs(),
    }
}


fn worker_loop(
    id: usize,
    job_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    result_tx: mpsc::Sender<WebsiteStatus>,
    cfg: Arc<Config>,
    shutdown: Arc<AtomicBool>,
) {
    while !shutdown.load(Ordering::SeqCst) {
        let url = match job_rx.lock().unwrap().recv() {
            Ok(u) => u,
            Err(_) => break,
        };
        let status = check_site(&url, &cfg);
        if result_tx.send(status).is_err() {
            break;
        }
    }
    eprintln!("worker-{id} exiting");
}


fn run_once(urls: &[String], cfg: Arc<Config>, shutdown: Arc<AtomicBool>) -> Vec<WebsiteStatus> {
    let (job_tx, job_rx) = mpsc::channel::<String>();
    let job_rx = Arc::new(Mutex::new(job_rx)); 
    let (result_tx, result_rx) = mpsc::channel::<WebsiteStatus>();

    
    let mut handles = Vec::with_capacity(WORKER_THREADS);
    for i in 0..WORKER_THREADS {
        let rx = Arc::clone(&job_rx);
        let tx = result_tx.clone();
        let cfg_c = cfg.clone();
        let sd = shutdown.clone();
        handles.push(thread::spawn(move || worker_loop(i, rx, tx, cfg_c, sd)));
    }
    drop(result_tx);

    
    for u in urls {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        let _ = job_tx.send(u.clone());
    }
    drop(job_tx);

    
    let mut out = Vec::with_capacity(urls.len());
    for _ in 0..urls.len() {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        match result_rx.recv() {
            Ok(ws) => out.push(ws),
            Err(_) => break,
        }
    }

    
    for h in handles {
        let _ = h.join();
    }

    out
}


fn run_monitor(urls: Vec<String>, cfg: Config) {
    let cfg = Arc::new(cfg);
    let shutdown = Arc::new(AtomicBool::new(false));

    if PERIODIC_INTERVAL_SECS.is_some() {
        let sd = shutdown.clone();
        thread::spawn(move || {
            let _ = io::stdin().read(&mut [0u8; 1]);
            eprintln!("shutdown requested (stdin)");
            sd.store(true, Ordering::SeqCst);
        });
    }

    match PERIODIC_INTERVAL_SECS {
        None => {
            let results = run_once(&urls, cfg.clone(), shutdown.clone());
            for r in results {
                println!(
                    "{{\"url\":\"{}\",\"status\":{},\"response_time_ms\":{},\"timestamp_epoch_secs\":{}}}",
                    r.url,
                    match &r.status {
                        Ok(code) => code.to_string(),
                        Err(e) => format!("\"{}\"", e.replace('"', "'")),
                    },
                    r.response_time_ms,
                    r.timestamp_epoch_secs
                );
            }
        }
        Some(secs) => {
            let interval = Duration::from_secs(secs);
            while !shutdown.load(Ordering::SeqCst) {
                let results = run_once(&urls, cfg.clone(), shutdown.clone());
                for r in results.iter() {
                    println!(
                        "{{\"url\":\"{}\",\"status\":{},\"response_time_ms\":{},\"timestamp_epoch_secs\":{}}}",
                        r.url,
                        match &r.status {
                            Ok(code) => code.to_string(),
                            Err(e) => format!("\"{}\"", e.replace('"', "'")),
                        },
                        r.response_time_ms,
                        r.timestamp_epoch_secs
                    );
                }
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }
                thread::sleep(interval);
            }
        }
    }
}

fn main() {
    let urls = vec![
        "https://www.google.com".to_string(),
        "https://www.youtube.com".to_string(),
        "https://www.facebook.com".to_string(),
        "https://www.twitter.com".to_string(),
        "https://www.instagram.com".to_string(),
        "https://www.wikipedia.org".to_string(),
        "https://www.reddit.com".to_string(),
        "https://www.amazon.com".to_string(),
        "https://www.linkedin.com".to_string(),
        "https://www.netflix.com".to_string(),
        "https://www.yahoo.com".to_string(),
        "https://www.tiktok.com".to_string(),
        "https://www.pinterest.com".to_string(),
        "https://www.microsoft.com".to_string(),
        "https://www.apple.com".to_string(),
        "https://www.imgur.com".to_string(),
        "https://www.office.com".to_string(),
        "https://www.bing.com".to_string(),
        "https://www.paypal.com".to_string(),
        "https://www.dropbox.com".to_string(),
        "https://www.twitch.tv".to_string(),
        "https://www.stackoverflow.com".to_string(),
        "https://www.ebay.com".to_string(),
        "https://www.quora.com".to_string(),
        "https://www.medium.com".to_string(),
        "https://www.spotify.com".to_string(),
        "https://www.adobe.com".to_string(),
        "https://www.soundcloud.com".to_string(),
        "https://www.nytimes.com".to_string(),
        "https://www.bbc.com".to_string(),
        "https://www.cnn.com".to_string(),
        "https://www.airbnb.com".to_string(),
        "https://www.yelp.com".to_string(),
        "https://www.etsy.com".to_string(),
        "https://www.github.com".to_string(),
        "https://www.stackexchange.com".to_string(),
        "https://www.walmart.com".to_string(),
        "https://www.microsoftonline.com".to_string(),
        "https://www.mozilla.org".to_string(),
        "https://www.zoom.us".to_string(),
        "https://www.nike.com".to_string(),
        "https://www.salesforce.com".to_string(),
        "https://www.blogger.com".to_string(),
        "https://www.flickr.com".to_string(),
        "https://www.live.com".to_string(),
        "https://www.spotify.com".to_string(),
        "https://www.office365.com".to_string(),
        "https://www.paypal.com".to_string(),

        // error-testing website
        "https://expired.badssl.com/".to_string(),    // TLS error
        "https://httpbin.org/status/404".to_string(), // 404 error
    ];

    let cfg = Config {
        timeout: Duration::from_secs(REQUEST_TIMEOUT_SECS),
        max_retries: MAX_RETRIES,
    };

    run_monitor(urls, cfg);
}