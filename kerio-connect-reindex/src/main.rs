use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use walkdir::WalkDir;

const TARGET_DIR: &str = "/opt/kerio/mailserver/store/mail/";
const SERVICE_NAME: &str = "kerio-connect.service";

/// Checks if the program is running as root by calling `id -u`
fn check_root() {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .expect("Failed to execute id command");

    let uid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .unwrap_or(1);

    if uid != 0 {
        eprintln!("Error: Please run as root to ensure you have the necessary permissions.");
        exit(1);
    }
}

/// Starts or stops the systemd service and verifies the exit status
fn manage_service(action: &str) {
    println!("Attempting to {} {}...", action, SERVICE_NAME);
    let status = Command::new("systemctl")
        .arg(action)
        .arg(SERVICE_NAME)
        .status()
        .expect("Failed to execute systemctl");

    if !status.success() {
        eprintln!("Error: Failed to {} {}.", action, SERVICE_NAME);
        eprintln!("Aborting operation to prevent data corruption or inconsistent states.");
        exit(1);
    }
    println!("Success: {} {}ed.\n---", SERVICE_NAME, action);
}

fn main() {
    // 1. Check for root privileges
    check_root();

    // 2. Check if the target directory exists
    let target_path = Path::new(TARGET_DIR);
    if !target_path.exists() || !target_path.is_dir() {
        eprintln!("Error: Directory {} does not exist.", TARGET_DIR);
        exit(1);
    }

    // 3. Stop the Kerio service
    manage_service("stop");

    println!("Scanning {} and renaming files in parallel...", TARGET_DIR);

    // 4. Sequentially walk the directory and collect target files
    // Doing the disk-read sequentially prevents I/O thrashing during discovery
    let mut files_to_rename: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(TARGET_DIR).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name == "index.fld" || file_name == "search.fld" {
                    files_to_rename.push(path.to_path_buf());
                }
            }
        }
    }

    // 5. Process the renames in parallel using Rayon
    files_to_rename.par_iter().for_each(|path| {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            let new_name = file_name.replace(".fld", ".bad");
            let mut new_path = path.to_path_buf();
            new_path.set_file_name(new_name);

            match fs::rename(path, &new_path) {
                Ok(_) => println!("Renamed: {} -> {}", path.display(), new_path.display()),
                Err(e) => eprintln!("Error renaming {}: {}", path.display(), e),
            }
        }
    });

    println!("---");
    
    // 6. Start the Kerio service
    manage_service("start");
    
    println!("Procedure completed successfully. Kerio Connect is rebuilding the indexes.");
}