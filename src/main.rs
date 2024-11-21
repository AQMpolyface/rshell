use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt; // For Unix-specific permission checks
use std::path::Path;
use std::process::exit;
use std::process::Command;
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
                //let input_borrow = &line;
                let inputsplit: Vec<&str> = line.split_whitespace().collect();
                let arguments: &[&str] = &inputsplit[1..];
                // println!("{:?}", input_borrow);
                let path_array: Vec<String> = match env::var("PATH") {
                    Ok(path) => path.split(':').map(String::from).collect(),
                    Err(_) => {
                        eprintln!("Failed to get PATH variable");
                        vec![]
                    }
                };
                //println!("{:?}", path_array);
                let mut command_found = false;
                for paths in path_array {
                    if check_file_exists_in_sbin(&inputsplit[0], &paths) {
                        let output = Command::new(inputsplit[0])
                            .args(arguments)
                            .output()
                            .expect("Failed to execute command");

                        println!("{}", String::from_utf8_lossy(&output.stdout));
                        command_found = true;
                        break;
                    }
                }

                if !command_found {
                    println!("The binary '{}' does not exist in PATH.", line);
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
    // Build the path in /sbin to check existence
    let final_path = [dir, "/", command_base].join("");
    let bin_path = Path::new(&final_path);
    // Check if the file exists
    //
    //println!("{}", final_path);
    return bin_path.exists();
}
