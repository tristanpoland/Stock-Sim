use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// Define the structure for our TOML configuration file
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    initial_amount: Decimal,
    weeks: u32,
    gains: Vec<Decimal>,
}

// Define the command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to a TOML configuration file. If not specified, the program looks for a 'config.toml' in the current directory.
    #[clap(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let config_path = args.config.unwrap_or_else(|| PathBuf::from("config.toml"));

    // Read the configuration from the TOML file
    let config = match read_config(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading configuration from {:?}: {}", config_path, e);
            return;
        }
    };

    let mut total_amount = config.initial_amount;
    let num_stocks = config.gains.len();

    if num_stocks == 0 {
        eprintln!("Error: The config file must contain at least one stock gain.");
        return;
    }

    println!(
        "Simulating {} weeks with an initial amount of {} from {:?}...",
        config.weeks, config.initial_amount, config_path
    );

    for week in 1..=config.weeks {
        let stock_index = ((week - 1) as usize) % num_stocks;
        let weekly_gain = config.gains[stock_index];

        total_amount += weekly_gain;
    }

    let total_gained = total_amount - config.initial_amount;
    let percentage_gain = (total_gained / config.initial_amount) * Decimal::from(100);

    println!("\n--- Results ---");
    println!("Final Total Amount: {:.2}", total_amount);
    println!("Total Amount Gained: {:.2}", total_gained);
    println!("Percentage Gain: {:.2}%", percentage_gain);
}

// Helper function to read the TOML file
fn read_config(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
