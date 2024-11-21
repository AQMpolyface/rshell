use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::exit;
use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    let blue_prompt = "\x1b[34mRshell> \x1b[0m";
    let mut rl = rustyline::DefaultEditor::new().unwrap();

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

                let mut command_found = false;
                for path in path_array {
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

fn check_file_exists_in_sbin(command_base: &str, dir: &str) -> bool {
    let final_path = [dir, "/", command_base].join("");
    let bin_path = Path::new(&final_path);
    bin_path.exists()
}
