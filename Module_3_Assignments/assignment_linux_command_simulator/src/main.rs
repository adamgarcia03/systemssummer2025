use std::process::Command;
use std::io;
use std::fs::OpenOptions;
use std::io::Write;
use std::fs::File;
use std::io::{BufReader, BufRead};



struct LinuxAgent {
    path: String,
}

impl LinuxAgent {
    fn new() -> LinuxAgent {
        let file_path = "command_history.txt";
        // Create the file if it doesn't exist
        File::create(file_path).expect("Unable to create history file.");
        LinuxAgent {
            path: file_path.to_string(),
        }
    }

    fn executing_os_commands_linux(&self, command_full: &str) {
        let mut parts = command_full.trim().split_whitespace();
        let command = match parts.next() {
            Some(cmd) => cmd,
            None => return,
        };

        let args: Vec<&str> = parts.collect();

        let output = Command::new(command)
            .args(&args)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let result = format!(
                    "\n$ {}\n[stdout]\n{}\n[stderr]\n{}\n---\n",
                    command_full, stdout, stderr
                );
                println!("{}", result);
                self.save_results(result);
            }
            Err(e) => {
                let error_msg = format!(
                    "\n$ {}\n[error] Failed to execute command: {}\n---\n",
                    command_full, e
                );
                println!("{}", error_msg);
                self.save_results(error_msg);
            }
        }
    }

    fn accept_linux_command_from_user() -> String {
        print!("Enter a Linux command (or type 'exit'): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input.");
        input.trim().to_string()
    }

    fn save_results(&self, content: String) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.path)
            .expect("Unable to open file.");
        file.write_all(content.as_bytes()).expect("Unable to write to file.");
    }

    fn show_results(&self) {
        println!("\n==== Command History ====");
        let file = File::open(&self.path).expect("Unable to open file.");
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(l) = line {
                println!("{}", l);
            }
        }
        println!("==========================");
    }
}

fn main() {
    let agent = LinuxAgent::new();

    loop {
        let command = LinuxAgent::accept_linux_command_from_user();
        if command.eq_ignore_ascii_case("exit") {
            break;
        }
        agent.executing_os_commands_linux(&command);
    }

    agent.show_results();
}