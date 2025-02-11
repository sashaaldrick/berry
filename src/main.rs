use clap::{CommandFactory, Parser};

const ASCII_ART: &str = r#"
   __  ___     ________    ____
  /  |/  /_  _/ ____/ /   /  _/
 / /|_/ / / / / /   / /    / /  
/ /  / / /_/ / /___/ /____/ /   
/_/  /_/\__, /\____/_____/___/  
       /____/                    
"#;

/// A modern CLI tool for project setup and management
#[derive(Parser)]
#[command(name = "mycli")]
#[command(author = "Your Name <your.email@example.com>")]
#[command(version = "0.1.0")]
#[command(about = format!("{}\nA modern CLI tool for project setup and management", ASCII_ART))]
struct Cli {
    /// Optional name to operate on
    #[arg(short, long)]
    name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Handle the name argument
    if let Some(name) = cli.name {
        println!("Hello, {}!", name);
    }
}
