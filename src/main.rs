use clap::Parser;
use std::path::PathBuf;

mod dsl;
mod yahoo_finance;
mod simulator;

use dsl::StockDSL;
use simulator::Simulator;


// Define the command-line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to a .stock DSL file. If not specified, the program looks for a 'Test.stock' in the current directory.
    #[clap(short, long, value_name = "FILE")]
    stock_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let stock_file_path = args.stock_file.unwrap_or_else(|| PathBuf::from("Test.stock"));

    // Parse the DSL file
    let dsl = match StockDSL::parse_file(&stock_file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing stock file {:?}: {}", stock_file_path, e);
            return;
        }
    };

    println!("Stock Simulator - Processing {:?}\n", stock_file_path);
    println!("Investment amounts: {:?}", dsl.invest_amounts);
    println!("Time frames: {:?}", dsl.time_frames);
    println!("Investments: {:?}", dsl.investments.keys().collect::<Vec<_>>());
    println!("Patterns: {:?}", dsl.patterns.keys().collect::<Vec<_>>());
    println!("Tests to run: {:?}\n", dsl.tests);

    // Create simulator and run simulations
    let mut simulator = Simulator::new();
    
    println!("Fetching stock data from Yahoo Finance...");
    match simulator.run_simulations(&dsl).await {
        Ok(results) => {
            Simulator::print_results(&results);
        }
        Err(e) => {
            eprintln!("Error running simulations: {}", e);
        }
    }
}

