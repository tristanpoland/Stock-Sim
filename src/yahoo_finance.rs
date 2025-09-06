use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    pub symbol: String,
    pub current_price: Decimal,
    pub historical_prices: Vec<HistoricalPrice>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPrice {
    pub date: DateTime<Utc>,
    pub close: Decimal,
    pub volume: u64,
}

#[derive(Debug, Deserialize)]
struct YahooQuoteResponse {
    #[serde(rename = "quoteResponse")]
    quote_response: QuoteResponse,
}

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    result: Vec<QuoteResult>,
}

#[derive(Debug, Deserialize)]
struct QuoteResult {
    symbol: String,
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: f64,
}

pub struct YahooFinanceClient {
    client: Client,
    cache: HashMap<String, StockData>,
}

impl YahooFinanceClient {
    pub fn new() -> Self {
        YahooFinanceClient {
            client: Client::new(),
            cache: HashMap::new(),
        }
    }

    pub async fn get_stock_data(&mut self, symbol: &str) -> Result<&StockData, Box<dyn Error + Send + Sync>> {
        // Check cache first
        let use_cache = if let Some(cached_data) = self.cache.get(symbol) {
            // Use cache if data is less than 1 hour old
            Utc::now().signed_duration_since(cached_data.fetched_at) < Duration::hours(1)
        } else {
            false
        };

        if use_cache {
            return Ok(self.cache.get(symbol).unwrap());
        }

        // Fetch fresh data
        let stock_data = self.fetch_stock_data(symbol).await?;
        self.cache.insert(symbol.to_string(), stock_data);

        Ok(self.cache.get(symbol).unwrap())
    }

    async fn fetch_stock_data(&self, symbol: &str) -> Result<StockData, Box<dyn Error + Send + Sync>> {
        // Get current price using Yahoo Finance v8 API
        let quote_url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1y",
            symbol
        );

        let response = self.client
            .get(&quote_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;

        let response_text = response.text().await?;
        let chart_data: ChartResponse = serde_json::from_str(&response_text)?;

        if chart_data.chart.result.is_empty() {
            return Err(format!("No data found for symbol: {}", symbol).into());
        }

        let result = &chart_data.chart.result[0];
        let meta = &result.meta;
        let current_price = Decimal::try_from(meta.regular_market_price)?;

        // Extract historical data
        let mut historical_prices = Vec::new();
        if let (Some(timestamps), Some(quotes)) = (&result.timestamp, &result.indicators.quote.get(0)) {
            if let Some(closes) = &quotes.close {
                for (i, &timestamp) in timestamps.iter().enumerate() {
                    if let Some(close) = closes.get(i).and_then(|&c| c) {
                        let date = DateTime::from_timestamp(timestamp as i64, 0)
                            .unwrap_or_else(|| Utc::now());
                        let close_decimal = Decimal::try_from(close)?;
                        
                        historical_prices.push(HistoricalPrice {
                            date,
                            close: close_decimal,
                            volume: quotes.volume.as_ref()
                                .and_then(|v| v.get(i))
                                .and_then(|&vol| vol)
                                .unwrap_or(0.0) as u64,
                        });
                    }
                }
            }
        }

        Ok(StockData {
            symbol: symbol.to_string(),
            current_price,
            historical_prices,
            fetched_at: Utc::now(),
        })
    }

    pub fn calculate_annual_return(&self, symbol: &str) -> Result<Decimal, Box<dyn Error + Send + Sync>> {
        let stock_data = self.cache.get(symbol)
            .ok_or(format!("No cached data for symbol: {}", symbol))?;

        if stock_data.historical_prices.len() < 2 {
            return Ok(Decimal::ZERO);
        }

        // Sort prices by date to ensure proper chronological order
        let mut sorted_prices = stock_data.historical_prices.clone();
        sorted_prices.sort_by(|a, b| a.date.cmp(&b.date));

        if sorted_prices.len() < 2 {
            return Ok(Decimal::ZERO);
        }

        let earliest_price = sorted_prices.first().unwrap().close;
        let latest_price = sorted_prices.last().unwrap().close;
        
        if earliest_price <= Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        // Calculate the time span in years
        let time_span_days = (sorted_prices.last().unwrap().date - sorted_prices.first().unwrap().date).num_days();
        let years = Decimal::try_from(time_span_days as f64 / 365.25)?;
        
        if years <= Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        // Calculate annualized return: (ending_value / starting_value)^(1/years) - 1
        let total_return = latest_price / earliest_price;
        
        // For realistic simulation, cap extreme returns and use a more conservative approach
        let capped_return = if total_return > Decimal::from(10) {
            // Cap at 10x (900% total return) to avoid unrealistic scenarios
            Decimal::from(10)
        } else if total_return < Decimal::try_from(0.1)? {
            // Floor at 0.1x (-90% total return) 
            Decimal::try_from(0.1)?
        } else {
            total_return
        };

        // Calculate annualized return
        // For simplicity, use logarithmic approximation for reasonable returns
        let annual_return = if capped_return > Decimal::ONE {
            // Positive return: use conservative growth estimate
            let excess_return = capped_return - Decimal::ONE;
            excess_return / years
        } else {
            // Negative return: linear approximation
            (capped_return - Decimal::ONE) / years
        };

        // Cap annual returns to realistic long-term market bounds (-30% to +40%)
        // Even the best performing stocks rarely sustain >40% annually over decades
        let realistic_return = if annual_return > Decimal::try_from(0.4)? {
            Decimal::try_from(0.4)? // Cap at 40% annual return for sustainability
        } else if annual_return < Decimal::try_from(-0.3)? {
            Decimal::try_from(-0.3)? // Floor at -30% annual return
        } else {
            annual_return
        };

        Ok(realistic_return)
    }
}

#[derive(Debug, Deserialize)]
struct ChartResponse {
    chart: Chart,
}

#[derive(Debug, Deserialize)]
struct Chart {
    result: Vec<ChartResult>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    meta: Meta,
    timestamp: Option<Vec<u32>>,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct Meta {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: f64,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
}

#[derive(Debug, Deserialize)]
struct Quote {
    close: Option<Vec<Option<f64>>>,
    volume: Option<Vec<Option<f64>>>,
}