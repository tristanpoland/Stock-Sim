use crate::dsl::{StockDSL, TimeFrame, TimeUnit};
use crate::yahoo_finance::YahooFinanceClient;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
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

        // Calculate total time period in years for realistic compounding
        let total_years = match time_frame.unit {
            TimeUnit::Days => Decimal::try_from(time_frame.duration as f64 / 365.25)?,
            TimeUnit::Weeks => Decimal::try_from(time_frame.duration as f64 / 52.0)?,
            TimeUnit::Years => Decimal::from(time_frame.duration),
        };

        // Simulate weekly trading but apply realistic annual returns
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

            // For the first trade, record the initial investment details
            if week == 1 {
                let shares_to_buy = current_amount / stock_price;
                trades.push(Trade {
                    week,
                    company: company_name.clone(),
                    price: stock_price,
                    shares_bought: shares_to_buy,
                    amount_invested: current_amount,
                });
            }
        }

        // Apply realistic compound growth over the entire time period
        if total_years > Decimal::ZERO {
            // Get a weighted average annual return from all stocks in the pattern
            let mut total_weighted_return = Decimal::ZERO;
            let mut total_weight = Decimal::ZERO;
            
            for company_name in pattern {
                if let Some(investment) = investments.values().find(|inv| inv.name == *company_name) {
                    let annual_return = self.yahoo_client.calculate_annual_return(&investment.ticker)?;
                    total_weighted_return += annual_return;
                    total_weight += Decimal::ONE;
                }
            }
            
            let avg_annual_return = if total_weight > Decimal::ZERO {
                total_weighted_return / total_weight
            } else {
                Decimal::ZERO
            };

            // Apply compound growth with realistic bounds
            let growth_factor = if total_years <= Decimal::from(5) {
                // For periods ≤ 5 years, allow normal compound growth
                let annual_multiplier = Decimal::ONE + avg_annual_return;
                let mut compound_factor = Decimal::ONE;
                let whole_years = total_years.floor();
                
                for _ in 0..whole_years.to_u32().unwrap_or(0) {
                    compound_factor *= annual_multiplier;
                }
                
                let fractional_year = total_years - whole_years;
                if fractional_year > Decimal::ZERO {
                    compound_factor *= Decimal::ONE + (avg_annual_return * fractional_year);
                }
                compound_factor
            } else {
                // For periods > 5 years, use more conservative modeling
                // Real market volatility and mean reversion make sustained high returns unlikely
                
                // Cap effective annual return for long periods (market reversion)
                let long_term_return = if avg_annual_return > Decimal::try_from(0.15)? {
                    // Even great stocks revert towards ~15% long-term
                    Decimal::try_from(0.15)?
                } else if avg_annual_return < Decimal::try_from(-0.1)? {
                    // Floor at -10% long-term (market recovery)
                    Decimal::try_from(-0.1)?
                } else {
                    avg_annual_return
                };
                
                // Use linear approximation for very long periods to avoid exponential explosion
                Decimal::ONE + (long_term_return * total_years)
            };
            
            current_amount *= growth_factor;
        }

        let total_gain = current_amount - initial_amount;
        let percentage_gain = if initial_amount > Decimal::ZERO {
            match (total_gain / initial_amount).checked_mul(Decimal::from(100)) {
                Some(gain) => gain,
                None => {
                    // Handle overflow in percentage calculation
                    if total_gain >= Decimal::ZERO {
                        Decimal::try_from(999999f64)? // Cap at 999,999% gain
                    } else {
                        Decimal::try_from(-100f64)? // Cap at 100% loss
                    }
                }
            }
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
                for trade in result.trades.iter().take(5) {
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