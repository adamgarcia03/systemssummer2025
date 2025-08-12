use serde::Deserialize; 
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ureq;

trait Pricing {
    fn fetch_price(&self) -> Result<(String, f64), String>;
    fn save_to_file(&self, name: &str, price: f64) -> Result<(), String> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_secs();

        let filename = format!(
            "{}.txt",
            name.to_lowercase()
                .replace(' ', "_")
                .replace("(", "")
                .replace(")", "")
        );

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&filename)
            .map_err(|e| format!("File open error ({}): {}", filename, e))?;

        writeln!(file, "{},{:.2}", ts, price)
            .map_err(|e| format!("Write error ({}): {}", filename, e))?;

        Ok(())
    }
}

// --- Bitcoin ---
struct Bitcoin;

#[derive(Deserialize)]
struct CoinGeckoPrice {
    usd: f64,
}

impl Pricing for Bitcoin {
    fn fetch_price(&self) -> Result<(String, f64), String> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
        let resp = ureq::get(url)
            .call()
            .map_err(|e| format!("Request error (CoinGecko BTC): {}", e))?;
        let data: HashMap<String, CoinGeckoPrice> = resp
            .into_json()
            .map_err(|e| format!("Parse error (CoinGecko BTC): {}", e))?;
        data.get("bitcoin")
            .map(|p| ("Bitcoin".to_string(), p.usd))
            .ok_or_else(|| "No bitcoin data found in response".to_string())
    }
}

// --- Ethereum ---
struct Ethereum;

impl Pricing for Ethereum {
    fn fetch_price(&self) -> Result<(String, f64), String> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
        let resp = ureq::get(url)
            .call()
            .map_err(|e| format!("Request error (CoinGecko ETH): {}", e))?;
        let data: HashMap<String, CoinGeckoPrice> = resp
            .into_json()
            .map_err(|e| format!("Parse error (CoinGecko ETH): {}", e))?;
        data.get("ethereum")
            .map(|p| ("Ethereum".to_string(), p.usd))
            .ok_or_else(|| "No ethereum data found in response".to_string())
    }
}

// --- S&P 500 ---
struct SP500;

#[derive(Deserialize)]
struct YahooChartResponse {
    chart: YahooChart,
}

#[derive(Deserialize)]
struct YahooChart {
    result: Vec<YahooResult>,
}

#[derive(Deserialize)]
struct YahooResult {
    meta: YahooMeta,
}

#[derive(Deserialize)]
struct YahooMeta {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: f64,
}

impl Pricing for SP500 {
    fn fetch_price(&self) -> Result<(String, f64), String> {
        let url = "https://query2.finance.yahoo.com/v8/finance/chart/%5EGSPC";
        let resp = ureq::get(url)
            .call()
            .map_err(|e| format!("Request error (Yahoo Finance ^GSPC): {}", e))?;

        let data: YahooChartResponse = resp
            .into_json()
            .map_err(|e| format!("Parse error (Yahoo Finance ^GSPC): {}", e))?;

        if let Some(first_result) = data.chart.result.first() {
            Ok((
                "S&P 500 (^GSPC)".to_string(),
                first_result.meta.regular_market_price,
            ))
        } else {
            Err("No data found for ^GSPC".to_string())
        }
    }
}

fn main() {
    let assets: Vec<Box<dyn Pricing>> =
        vec![Box::new(Bitcoin), Box::new(Ethereum), Box::new(SP500)];

    println!("Financial Data Fetcher");
    loop {
        println!("\nFetching prices...");
        for asset in &assets {
            match asset.fetch_price() {
                Ok((name, price)) => {
                    println!("{}: ${:.2}", name, price);
                    match asset.save_to_file(&name, price) {
                        Ok(_) => println!("Saved to file."),
                        Err(e) => eprintln!("Save error: {}", e),
                    }
                }
                Err(e) => eprintln!("Fetch error: {}", e),
            }
        }
        println!("Waiting 10 seconds...");
        thread::sleep(Duration::from_secs(10));
    }
}
