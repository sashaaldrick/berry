use clap::{Parser, Subcommand};
use git2::Repository;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const ASCII_ART: &str = r#"
    ____                        
   / __ )___  ____________  __
  / __  / _ \/ ___/ ___/ / / /
 / /_/ /  __/ /  / /  / /_/ / 
/_____/\___/_/  /_/   \__, /  
                     /____/    
"#;

const CHECK_MARK: &str = "✓";
const CROSS_MARK: &str = "✗";

/// A modern CLI tool for project setup and management
#[derive(Parser)]
#[command(name = "berry")]
#[command(author = "Your Name <your.email@example.com>")]
#[command(version = "0.1.0")]
#[command(about = format!("{}\nA modern CLI tool for project setup and management", ASCII_ART))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new folder
    New {
        /// Name of the folder to create
        name: String,
    },
}

/// Get command version output
fn get_command_version(command: &str, args: &[&str]) -> Option<String> {
    Command::new(command)
        .args(args)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
}

/// Check if Rust is installed and get its version
fn check_rust() -> Result<String, String> {
    if let Some(version) = get_command_version("rustc", &["--version"]) {
        let version = version.trim().to_string();
        // Extract just the version number
        let version = version
            .replace("rustc ", "")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        Ok(format!("Rust v{}", version))
    } else {
        Err(
            "Rust not found. To install, visit: https://www.rust-lang.org/tools/install"
                .to_string(),
        )
    }
}

/// Check if Foundry is installed and get its version
fn check_foundry() -> Result<String, String> {
    if let Some(version) = get_command_version("forge", &["--version"]) {
        let version = version.trim().to_string();
        // Extract just the version number
        let version = version.split_whitespace().nth(1).unwrap_or("").to_string();
        Ok(format!("Foundry v{}", version))
    } else {
        Err("Foundry not found. To install, visit: https://book.getfoundry.sh/getting-started/installation".to_string())
    }
}

/// Check if RISC0 is installed and get its version
fn check_risc0() -> Result<String, String> {
    if let Some(version) = get_command_version("cargo", &["risczero", "--version"]) {
        let version = version.trim().to_string();
        // Extract just the version number
        let version = version.split_whitespace().nth(1).unwrap_or("").to_string();
        // Check if version starts with 1.2
        if version.contains("1.2") {
            Ok(format!("RISC0 v{}", version))
        } else {
            Err(format!(
                "Unsupported RISC0 version: {}. Version 1.2.x is required",
                version
            ))
        }
    } else {
        Err(
            "RISC0 not found. To install, visit: https://dev.risczero.com/api/zkvm/install"
                .to_string(),
        )
    }
}

/// Run a git command in the specified directory
fn run_git_command(dir: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}

/// Set up sparse checkout for the repository
fn setup_sparse_checkout(dir: &str) -> Result<(), String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Setting up sparse checkout...");
    pb.enable_steady_tick(Duration::from_millis(100));

    // Initialize sparse checkout
    run_git_command(dir, &["sparse-checkout", "init", "--cone"])?;
    run_git_command(dir, &["sparse-checkout", "set", "examples/erc20-counter"])?;

    pb.finish_with_message(format!("{} Sparse checkout completed", CHECK_MARK));
    Ok(())
}

/// Clone the RISC0 repository
fn clone_repository(name: &str, _branch: &str) -> Result<(), git2::Error> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Cloning RISC0 repository into {}...", name));
    pb.enable_steady_tick(Duration::from_millis(100));

    // Clone with specific branch
    Repository::clone_recurse("https://github.com/risc0/risc0-ethereum.git", name)?;

    pb.finish_with_message(format!("{} Repository cloned successfully", CHECK_MARK));
    Ok(())
}

/// Move files from erc20-counter to root and clean up
fn setup_project_files(dir: &str) -> Result<(), String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Setting up project files...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let dir_path = PathBuf::from(dir);
    let erc20_path = dir_path.join("examples").join("erc20-counter");
    let examples_path = dir_path.join("examples");

    // Move erc20-counter out of examples/
    if erc20_path.exists() {
        fs::rename(&erc20_path, dir_path.join("erc20-counter"))
            .map_err(|e| format!("Failed to move erc20-counter: {}", e))?;
    }

    // Remove examples directory
    if examples_path.exists() {
        fs::remove_dir_all(examples_path)
            .map_err(|e| format!("Failed to remove examples directory: {}", e))?;
    }

    // Delete files in root directory
    for entry in fs::read_dir(&dir_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove file {}: {}", path.display(), e))?;
        }
    }

    // Move all contents from erc20-counter to root
    let temp_counter_path = dir_path.join("erc20-counter");
    if temp_counter_path.exists() {
        for entry in fs::read_dir(&temp_counter_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let file_name = path.file_name().ok_or("Invalid file name")?;
            let target_path = dir_path.join(file_name);
            fs::rename(&path, &target_path)
                .map_err(|e| format!("Failed to move {}: {}", path.display(), e))?;
        }

        // Remove the now-empty erc20-counter directory
        fs::remove_dir_all(temp_counter_path)
            .map_err(|e| format!("Failed to remove erc20-counter directory: {}", e))?;
    }

    pb.finish_with_message(format!("{} Project files set up successfully", CHECK_MARK));
    Ok(())
}

/// Update dependencies in Cargo.toml files
fn update_cargo_dependencies(dir: &str) -> Result<(), String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Updating Cargo.toml files...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let dir_path = PathBuf::from(dir);
    visit_cargo_files(&dir_path, &pb)?;

    pb.finish_with_message(format!(
        "{} Cargo.toml files updated successfully",
        CHECK_MARK
    ));
    Ok(())
}

fn visit_cargo_files(dir: &Path, pb: &ProgressBar) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            visit_cargo_files(&path, pb)?;
        } else if path.file_name().map_or(false, |n| n == "Cargo.toml") {
            pb.set_message(format!("Updating {}", path.display()));
            update_cargo_file(&path)?;
        }
    }

    Ok(())
}

fn update_cargo_file(path: &Path) -> Result<(), String> {
    // Read the file content
    let mut content = String::new();
    let mut file =
        fs::File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    // Check if file is in apps directory
    let is_apps = path.to_string_lossy().contains("/apps/");

    // Update dependencies using regex-like replacements
    let mut updated = content
        .replace(
            "risc0-build-ethereum = { path = \"../../build\" }",
            "risc0-build-ethereum = { git = \"https://github.com/risc0/risc0-ethereum\", branch = \"release-1.3\" }",
        )
        .replace(
            "risc0-ethereum-contracts = { path = \"../../contracts\" }",
            "risc0-ethereum-contracts = { git = \"https://github.com/risc0/risc0-ethereum\", branch = \"release-1.3\" }",
        );

    // Different handling for files in apps/ directory
    if is_apps {
        updated = updated.replace(
            "risc0-steel = { path = \"../../crates/steel\" }",
            "risc0-steel = { git = \"https://github.com/risc0/risc0-ethereum\", branch = \"release-1.3\", features = [\"host\"] }",
        );
    } else {
        updated = updated.replace(
            "risc0-steel = { path = \"../../crates/steel\" }",
            "risc0-steel = { git = \"https://github.com/risc0/risc0-ethereum\", branch = \"release-1.3\" }",
        );
    }

    // Write back to file
    let mut file = fs::File::create(path)
        .map_err(|e| format!("Failed to open {} for writing: {}", path.display(), e))?;
    file.write_all(updated.as_bytes())
        .map_err(|e| format!("Failed to write to {}: {}", path.display(), e))?;

    Ok(())
}

/// Update foundry.toml configuration
fn update_foundry_config(dir: &str) -> Result<(), String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Updating foundry.toml...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let foundry_path = PathBuf::from(dir).join("foundry.toml");
    if !foundry_path.exists() {
        pb.finish_with_message(format!("{} foundry.toml not found", CROSS_MARK));
        return Ok(());
    }

    // Read the file content
    let mut content = String::new();
    let mut file =
        fs::File::open(&foundry_path).map_err(|e| format!("Failed to open foundry.toml: {}", e))?;
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read foundry.toml: {}", e))?;

    // Update the libs path and add auto_detect_remappings = false
    let updated = content
        .replace(
            "libs = [\"../../lib\", \"../../contracts/src\"]",
            "libs = [\"lib\"]",
        )
        .replace(
            "[profile.default]",
            "[profile.default]\nauto_detect_remappings = false",
        );

    // Write back to file
    let mut file = fs::File::create(&foundry_path)
        .map_err(|e| format!("Failed to open foundry.toml for writing: {}", e))?;
    file.write_all(updated.as_bytes())
        .map_err(|e| format!("Failed to write to foundry.toml: {}", e))?;

    pb.finish_with_message(format!("{} foundry.toml updated successfully", CHECK_MARK));
    Ok(())
}

/// Set up Git submodules
fn setup_git_submodules(dir: &str) -> Result<(), String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Setting up Git submodules...");
    pb.enable_steady_tick(Duration::from_millis(100));

    // Clean up existing lib directory
    let lib_path = PathBuf::from(dir).join("lib");
    if lib_path.exists() {
        fs::remove_dir_all(&lib_path)
            .map_err(|e| format!("Failed to remove lib directory: {}", e))?;
    }
    fs::create_dir_all(&lib_path).map_err(|e| format!("Failed to create lib directory: {}", e))?;

    // Remove existing .git directory to start fresh
    let git_path = PathBuf::from(dir).join(".git");
    if git_path.exists() {
        fs::remove_dir_all(&git_path)
            .map_err(|e| format!("Failed to remove .git directory: {}", e))?;
    }

    // Initialize new git repository
    run_git_command(dir, &["init"])?;

    // Initialize and add submodules
    run_git_command(dir, &["submodule", "init"])?;

    // Add forge-std
    pb.set_message("Adding forge-std submodule...");
    run_git_command(
        dir,
        &[
            "submodule",
            "add",
            "https://github.com/foundry-rs/forge-std",
            "lib/forge-std",
        ],
    )?;

    // Add OpenZeppelin contracts
    pb.set_message("Adding OpenZeppelin contracts submodule...");
    run_git_command(
        dir,
        &[
            "submodule",
            "add",
            "https://github.com/OpenZeppelin/openzeppelin-contracts",
            "lib/openzeppelin-contracts",
        ],
    )?;

    // Add RISC0 ethereum
    pb.set_message("Adding RISC0 ethereum submodule...");
    run_git_command(
        dir,
        &[
            "submodule",
            "add",
            "-b",
            "release-1.3",
            "https://github.com/risc0/risc0-ethereum",
            "lib/risc0-ethereum",
        ],
    )?;

    // Update all submodules recursively
    pb.set_message("Updating submodules...");
    run_git_command(
        dir,
        &["submodule", "update", "--init", "--recursive", "--quiet"],
    )?;

    // Reset git state
    run_git_command(dir, &["reset"])?;

    pb.finish_with_message(format!("{} Git submodules set up successfully", CHECK_MARK));
    Ok(())
}

/// Update remappings.txt configuration
fn update_remappings(dir: &str) -> Result<(), String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Updating remappings.txt...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let remappings_path = PathBuf::from(dir).join("remappings.txt");
    if !remappings_path.exists() {
        pb.finish_with_message(format!("{} remappings.txt not found", CROSS_MARK));
        return Ok(());
    }

    // Read the file content
    let mut content = String::new();
    let mut file = fs::File::open(&remappings_path)
        .map_err(|e| format!("Failed to open remappings.txt: {}", e))?;
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read remappings.txt: {}", e))?;

    // Update the remappings
    let mut updated = content
        .replace(
            "forge-std/=../../lib/forge-std/src/",
            "forge-std/=lib/forge-std/src/",
        )
        .replace(
            "openzeppelin/=../../lib/openzeppelin-contracts/",
            "openzeppelin/=lib/openzeppelin-contracts/",
        )
        .replace(
            "risc0/=../../contracts/src/",
            "risc0/=lib/risc0-ethereum/contracts/src/",
        );

    // Add OpenZeppelin contracts remapping if not present
    let oz_remapping = "openzeppelin-contracts/=lib/openzeppelin-contracts/contracts";
    if !updated.contains(oz_remapping) {
        if !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(oz_remapping);
        updated.push('\n');
    }

    // Write back to file
    let mut file = fs::File::create(&remappings_path)
        .map_err(|e| format!("Failed to open remappings.txt for writing: {}", e))?;
    file.write_all(updated.as_bytes())
        .map_err(|e| format!("Failed to write to remappings.txt: {}", e))?;

    pb.finish_with_message(format!(
        "{} remappings.txt updated successfully",
        CHECK_MARK
    ));
    Ok(())
}

/// Initialize a new project
fn init_project(name: &str) -> Result<(), String> {
    // Clone the repository
    clone_repository(name, "release-1.3").map_err(|e| e.to_string())?;

    // Switch to the release branch
    run_git_command(name, &["checkout", "release-1.3"])?;

    // Set up sparse checkout
    setup_sparse_checkout(name)?;

    // Set up project files
    setup_project_files(name)?;

    // Update Cargo.toml files
    update_cargo_dependencies(name)?;

    // Update foundry.toml
    update_foundry_config(name)?;

    // Set up Git submodules
    setup_git_submodules(name)?;

    // Update remappings.txt
    update_remappings(name)?;

    // Print success message
    println!("\n🫐 Project {} created successfully!", name);
    println!("\nNext steps:");
    println!("  1. cd {}", name);
    println!("  2. forge build");
    println!("  3. cargo test");

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => {
            let mut all_deps_ok = true;

            // Check Rust
            match check_rust() {
                Ok(version) => println!("{} {}", CHECK_MARK, version),
                Err(e) => {
                    println!("{} Rust: {}", CROSS_MARK, e);
                    all_deps_ok = false;
                }
            }

            // Check Foundry
            match check_foundry() {
                Ok(version) => println!("{} {}", CHECK_MARK, version),
                Err(e) => {
                    println!("{} Foundry: {}", CROSS_MARK, e);
                    all_deps_ok = false;
                }
            }

            // Check RISC0
            match check_risc0() {
                Ok(version) => println!("{} {}", CHECK_MARK, version),
                Err(e) => {
                    println!("{} RISC0: {}", CROSS_MARK, e);
                    all_deps_ok = false;
                }
            }

            if !all_deps_ok {
                return;
            }

            // Validate folder name is not empty
            if name.trim().is_empty() {
                eprintln!("{} Error: Folder name cannot be empty", CROSS_MARK);
                return;
            }

            // Check if folder already exists
            if Path::new(&name).exists() {
                eprintln!(
                    "{} Error: A file or directory named '{}' already exists",
                    CROSS_MARK, name
                );
                return;
            }

            // Initialize the project
            match init_project(&name) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{} Error initializing project: {}", CROSS_MARK, e);
                    // Clean up the directory if it was created
                    if Path::new(&name).exists() {
                        let _ = fs::remove_dir_all(&name);
                    }
                    return;
                }
            }
        }
    }
}
