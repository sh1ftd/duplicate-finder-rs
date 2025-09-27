mod duplicate_detector;
mod file_scanner;
mod hasher;
mod organizer;
#[cfg(test)]
mod tests;
mod workflow;

use crate::workflow::{WorkflowError, WorkflowSummary, execute};
use dialoguer::{Input, Select};
use std::error::Error;
use std::io;
use std::process;

fn main() {
    let root_path = match prompt_for_directory() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to determine directory to scan: {error}");
            process::exit(1);
        }
    };

    run_application(&root_path);
}

fn run_application(root_path: &str) {
    println!("Scanning directory: {root_path}");
    println!("Finding duplicate files...");

    match execute(root_path) {
        Ok(summary) => {
            let WorkflowSummary {
                files_scanned,
                duplicate_group_count,
                duplicates_found,
                index_path,
                index_content,
                index_read_error,
            } = summary;

            println!("Found {files_scanned} files to process");
            println!("Found {duplicate_group_count} groups of duplicate files");

            if duplicates_found {
                println!("Successfully organized duplicate files!");
                println!("Check the 'duplicates' folder for organized files.");
            } else {
                println!("No duplicate files found!");
            }

            println!();
            println!("=== COMPREHENSIVE DUPLICATE FILES INDEX ===");

            match (index_content, index_read_error) {
                (Some(content), _) => println!("{content}"),
                (None, Some(error)) => {
                    eprintln!("Warning: Could not read index file: {error}");
                    eprintln!(
                        "The index file should be available at: {}",
                        index_path.display()
                    );
                }
                (None, None) => {
                    eprintln!("Warning: Index file could not be read.");
                    eprintln!(
                        "The index file should be available at: {}",
                        index_path.display()
                    );
                }
            }

            println!("=== END OF INDEX ===");
            println!();

            println!("Press Enter to exit...");
            if let Err(error) = wait_for_enter() {
                eprintln!("Failed to wait for input: {error}");
            }
        }
        Err(error) => {
            handle_workflow_error(&error);
            process::exit(1);
        }
    }
}

fn handle_workflow_error(error: &WorkflowError) {
    eprintln!("{error}");
    if let Some(source) = error.source() {
        eprintln!("Caused by: {source}");
    }
}

fn prompt_for_directory() -> io::Result<String> {
    let options = ["Use current directory", "Provide custom directory path"];

    let selection = Select::new()
        .with_prompt("Select the directory to scan")
        .items(options)
        .default(0)
        .interact()
        .map_err(|error| io::Error::other(error.to_string()))?;

    match selection {
        0 => Ok(".".to_string()),
        1 => Input::new()
            .with_prompt("Enter the directory path to scan")
            .interact_text()
            .map_err(|error| io::Error::other(error.to_string())),
        _ => unreachable!("Select should only return indices for provided options"),
    }
}

fn wait_for_enter() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).map(|_| ())
}
