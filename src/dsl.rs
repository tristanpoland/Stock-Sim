use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct StockDSL {
    pub invest_amounts: Vec<Decimal>,
    pub time_frames: Vec<TimeFrame>,
    pub investments: HashMap<String, Investment>,
    pub patterns: HashMap<String, Vec<String>>,
    pub tests: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TimeFrame {
    pub duration: u32,
    pub unit: TimeUnit,
}

#[derive(Debug, Clone)]
pub enum TimeUnit {
    Days,
    Weeks,
    Years,
}

#[derive(Debug, Clone)]
pub struct Investment {
    pub ticker: String,
    pub name: String,
    pub price: Option<Decimal>, // Will be fetched from Yahoo Finance
}

impl StockDSL {
    pub fn new() -> Self {
        StockDSL {
            invest_amounts: Vec::new(),
            time_frames: Vec::new(),
            investments: HashMap::new(),
            patterns: HashMap::new(),
            tests: Vec::new(),
        }
    }

    pub fn parse_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut dsl = StockDSL::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "INVEST" => {
                    if parts.len() >= 2 {
                        dsl.parse_invest(&parts[1..])?;
                    }
                }
                "TIME" => {
                    if parts.len() >= 2 {
                        dsl.parse_time(&parts[1..])?;
                    }
                }
                "INVESTMENT" => {
                    if parts.len() >= 3 {
                        dsl.parse_investment(&parts[1..])?;
                    }
                }
                "PATTERN" => {
                    if parts.len() >= 3 {
                        dsl.parse_pattern(&parts[1..])?;
                    }
                }
                "TEST" => {
                    if parts.len() >= 2 {
                        dsl.tests.push(parts[1].to_string());
                    }
                }
                _ => {
                    // Ignore unrecognized commands
                }
            }
        }

        Ok(dsl)
    }

    fn parse_invest(&mut self, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let amounts_str = parts.join(" ");
        for amount_str in amounts_str.split(',') {
            let amount = amount_str.trim().parse::<Decimal>()?;
            self.invest_amounts.push(amount);
        }
        Ok(())
    }

    fn parse_time(&mut self, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let times_str = parts.join(" ");
        for time_str in times_str.split(',') {
            let time_str = time_str.trim();
            let time_frame = self.parse_time_frame(time_str)?;
            self.time_frames.push(time_frame);
        }
        Ok(())
    }

    fn parse_time_frame(&self, time_str: &str) -> Result<TimeFrame, Box<dyn std::error::Error>> {
        let len = time_str.len();
        if len < 2 {
            return Err("Invalid time format".into());
        }

        let (number_part, unit_part) = time_str.split_at(len - 1);
        let duration = number_part.parse::<u32>()?;
        
        let unit = match unit_part {
            "d" => TimeUnit::Days,
            "w" => TimeUnit::Weeks,
            "y" => TimeUnit::Years,
            _ => return Err(format!("Invalid time unit: {}", unit_part).into()),
        };

        Ok(TimeFrame { duration, unit })
    }

    fn parse_investment(&mut self, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if parts.len() >= 2 {
            let ticker = parts[0].to_string();
            let name = parts[1..].join(" ");
            
            let investment = Investment {
                ticker: ticker.clone(),
                name,
                price: None, // Will be fetched later
            };
            
            self.investments.insert(ticker, investment);
        }
        Ok(())
    }

    fn parse_pattern(&mut self, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if parts.len() >= 2 {
            let pattern_name = parts[0].to_string();
            let companies_str = parts[1..].join(" ");
            
            let mut companies = Vec::new();
            for company in companies_str.split(',') {
                companies.push(company.trim().to_string());
            }
            
            self.patterns.insert(pattern_name, companies);
        }
        Ok(())
    }
}