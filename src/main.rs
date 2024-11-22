use rustyline::error::ReadlineError;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::exit;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::{signal, sync::broadcast};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shutdown_rx = shutdown_signal().await;

    let work_handle = tokio::spawn(async move {
        tokio::select! {
            _ = shell() => {},
            _ = shutdown_rx.recv() => {
                println!("Work received shutdown signal");
            }
        }

        println!("Cleaning up...");
    });

    // Wait for work to complete
    work_handle.await?;
    println!("Shutdown complete");

    Ok(())
}

async fn shell() -> std::io::Result<()> {
    let blue_prompt = "\x1b[34mRshell> \x1b[0m";
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("Received Ctrl+C!");
    })
    .expect("Error setting Ctrl+C handler");

    loop {
        let input = rl.readline(blue_prompt);
        match input {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let inputsplit: Vec<&str> = line.split_whitespace().collect();
                let arguments: &[&str] = &inputsplit[1..];

                let path_array: Vec<String> = match env::var("PATH") {
                    Ok(path) => path.split(':').map(String::from).collect(),
                    Err(_) => {
                        eprintln!("Failed to get PATH variable");
                        vec![]
                    }
                };

                for _ in path_array {
                    match Command::new(inputsplit[0])
                        .args(arguments)
                        .stdout(Stdio::piped())
                        .spawn()
                    {
                        Ok(mut child) => {
                            if let Some(stdout) = child.stdout.take() {
                                let reader = BufReader::new(stdout);
                                for line in reader.lines() {
                                    println!("{}", line.unwrap());
                                }
                            }
                            break;
                        }
                        Err(e) => {
                            eprintln!("Command '{}' failed: {}", inputsplit[0], e);
                            break;
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!(
                    "{}",
                    "\x1b[31mPlease use ctrl + d if you want to exit \x1b[0m"
                );
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting...");
                exit(0);
            }
            Err(e) => {
                eprintln!("\x1b[31mError reading input: {}\x1b[0m", e);
                continue;
            }
        }
    }
}

async fn shutdown_signal() -> broadcast::Receiver<()> {
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("\nquitting...");
        let _ = shutdown_tx.send(());
    });

    shutdown_rx
}
