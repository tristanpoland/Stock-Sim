use crate::dsl::{StockDSL, TimeFrame, TimeUnit};
use crate::yahoo_finance::YahooFinanceClient;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct SimulationResult {
    pub pattern_name: String,
    pub initial_amount: Decimal,
    pub time_frame: TimeFrame,
    pub final_amount: Decimal,
    pub total_gain: Decimal,
    pub percentage_gain: Decimal,
    pub trades: Vec<Trade>,
}

#[derive(Debug)]
pub struct Trade {
    pub week: u32,
    pub company: String,
    pub price: Decimal,
    pub shares_bought: Decimal,
    pub amount_invested: Decimal,
}

pub struct Simulator {
    yahoo_client: YahooFinanceClient,
}

impl Simulator {
    pub fn new() -> Self {
        Simulator {
            yahoo_client: YahooFinanceClient::new(),
        }
    }

    pub async fn run_simulations(&mut self, dsl: &StockDSL) -> Result<Vec<SimulationResult>, Box<dyn Error + Send + Sync>> {
        let mut results = Vec::new();

        // Pre-fetch all stock data to populate cache
        for investment in dsl.investments.values() {
            self.yahoo_client.get_stock_data(&investment.ticker).await?;
        }

        // Run simulations for each test pattern
        for test_name in &dsl.tests {
            if let Some(pattern) = dsl.patterns.get(test_name) {
                // Run simulation for each combination of invest amount and time frame
                for &invest_amount in &dsl.invest_amounts {
                    for time_frame in &dsl.time_frames {
                        let result = self.simulate_pattern(
                            test_name,
                            pattern,
                            invest_amount,
                            time_frame,
                            &dsl.investments,
                        ).await?;
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    async fn simulate_pattern(
        &mut self,
        pattern_name: &str,
        pattern: &[String],
        initial_amount: Decimal,
        time_frame: &TimeFrame,
        investments: &HashMap<String, crate::dsl::Investment>,
    ) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let mut current_amount = initial_amount;
        let mut trades = Vec::new();

        let total_weeks = match time_frame.unit {
            TimeUnit::Days => (time_frame.duration + 6) / 7, // Round up to nearest week
            TimeUnit::Weeks => time_frame.duration,
            TimeUnit::Years => time_frame.duration * 52,
        };

        if pattern.is_empty() {
            return Err("Empty pattern".into());
        }

        for week in 1..=total_weeks {
            let company_index = ((week - 1) as usize) % pattern.len();
            let company_name = &pattern[company_index];

            // Find the investment by name (not ticker)
            let investment = investments.values()
                .find(|inv| inv.name == *company_name)
                .ok_or(format!("Investment not found for company: {}", company_name))?;

            // Get current stock price from cache
            let stock_data = self.yahoo_client.get_stock_data(&investment.ticker).await?;
            let stock_price = stock_data.current_price;

            // Calculate how many shares we can buy with current amount
            let shares_to_buy = current_amount / stock_price;
            let amount_invested = shares_to_buy * stock_price;

            trades.push(Trade {
                week,
                company: company_name.clone(),
                price: stock_price,
                shares_bought: shares_to_buy,
                amount_invested,
            });

            // Apply average gain based on historical data
            let avg_gain = self.yahoo_client.calculate_average_gain(&investment.ticker, 30)?; // 30-day average
            current_amount = amount_invested * (Decimal::ONE + avg_gain);
        }

        let total_gain = current_amount - initial_amount;
        let percentage_gain = if initial_amount > Decimal::ZERO {
            (total_gain / initial_amount) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        Ok(SimulationResult {
            pattern_name: pattern_name.to_string(),
            initial_amount,
            time_frame: time_frame.clone(),
            final_amount: current_amount,
            total_gain,
            percentage_gain,
            trades,
        })
    }

    pub fn print_results(results: &[SimulationResult]) {
        println!("\n=== STOCK SIMULATION RESULTS ===\n");

        for result in results {
            println!("Pattern: {}", result.pattern_name);
            println!("Initial Investment: ${:.2}", result.initial_amount);
            println!("Time Frame: {} {:?}", result.time_frame.duration, result.time_frame.unit);
            println!("Final Amount: ${:.2}", result.final_amount);
            println!("Total Gain: ${:.2}", result.total_gain);
            println!("Percentage Gain: {:.2}%", result.percentage_gain);
            println!("Number of Trades: {}", result.trades.len());
            
            if !result.trades.is_empty() {
                println!("Sample Trades:");
                for (i, trade) in result.trades.iter().take(5).enumerate() {
                    println!("  Week {}: {} @ ${:.2} ({:.4} shares)",
                        trade.week, trade.company, trade.price, trade.shares_bought);
                }
                if result.trades.len() > 5 {
                    println!("  ... and {} more trades", result.trades.len() - 5);
                }
            }
            println!("{}", "-".repeat(50));
        }

        // Summary statistics
        if !results.is_empty() {
            let best_result = results.iter()
                .max_by(|a, b| a.percentage_gain.partial_cmp(&b.percentage_gain).unwrap())
                .unwrap();
            
            let worst_result = results.iter()
                .min_by(|a, b| a.percentage_gain.partial_cmp(&b.percentage_gain).unwrap())
                .unwrap();

            println!("\n=== SUMMARY ===");
            println!("Best Performance: {} with {:.2}% gain", 
                best_result.pattern_name, best_result.percentage_gain);
            println!("Worst Performance: {} with {:.2}% gain", 
                worst_result.pattern_name, worst_result.percentage_gain);
        }
    }
}