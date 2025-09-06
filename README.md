# Stock Simulator

A Rust-based stock market simulation tool that uses a custom DSL (Domain Specific Language) to define investment strategies and test their performance using real Yahoo Finance data.

## Features

- **Custom DSL**: Define investment strategies using a simple `.stock` file format
- **Yahoo Finance Integration**: Fetch real stock price data for accurate simulations
- **Pattern Testing**: Create and test different investment patterns
- **Multiple Time Frames**: Test strategies across days, weeks, and years
- **Batch Testing**: Test multiple investment amounts and time frames simultaneously
- **VSCode Extension**: Syntax highlighting and language support for `.stock` files

## Installation

### Prerequisites
- Rust 1.70+ (2024 edition)
- Internet connection (for Yahoo Finance API calls)

### Building from Source

```bash
git clone https://github.com/yourusername/stock_simulator
cd stock_simulator
cargo build --release
```

## Usage

### Basic Usage

Run with default `Test.stock` file:
```bash
cargo run
```

Run with a specific `.stock` file:
```bash
cargo run -- -s my_strategy.stock
```

### DSL Syntax

Create a `.stock` file with the following syntax:

```stock
// Define investment amounts to test (in dollars)
INVEST 100,500,1000

// Define time frames to test
TIME 30d,12w,1y

// Define individual investments (ticker and display name)
INVESTMENT AAPL Apple
INVESTMENT MSFT Microsoft
INVESTMENT GOOGL Google

// Create patterns (sequences of investments)
PATTERN TechGrowth Apple,Microsoft,Google
PATTERN Conservative Apple,Apple,Microsoft

// Run tests on specific patterns
TEST TechGrowth
TEST Conservative
```

#### DSL Commands

- `INVEST <amounts>`: Comma-separated list of investment amounts in dollars
- `TIME <periods>`: Comma-separated list of time periods (format: `<number><unit>` where unit is `d`, `w`, or `y`)
- `INVESTMENT <ticker> <name>`: Define a stock investment with ticker symbol and display name
- `PATTERN <name> <investments>`: Create a named pattern of investments
- `TEST <pattern>`: Run simulation tests on a specific pattern

### Example Output

```
Stock Simulator - Processing "Test.stock"

Investment amounts: [10, 100, 1000]
Time frames: ["10d", "12w", "2y"]
Investments: ["AAPL", "MSFT"]
Patterns: ["MyPattern", "MyPattern2"]
Tests to run: ["MyPattern", "MyPattern2"]

Fetching stock data from Yahoo Finance...
Running simulations...

Results:
Pattern: MyPattern, Amount: $100, Time: 12w
  Initial: $100.00, Final: $125.50, Gain: 25.50%
```

## Project Structure

```
├── src/
│   ├── main.rs           # CLI entry point and argument parsing
│   ├── dsl.rs            # DSL parser and data structures
│   ├── simulator.rs      # Core simulation logic
│   └── yahoo_finance.rs  # Yahoo Finance API integration
├── vscode-extension/     # VSCode extension for .stock files
├── Test.stock           # Example DSL file
├── config.toml          # Configuration file
└── Cargo.toml          # Rust dependencies and metadata
```

## VSCode Extension

The project includes a VSCode extension that provides:
- Syntax highlighting for `.stock` files
- Language configuration for better editing experience

Install the extension by copying the `vscode-extension` folder to your VSCode extensions directory.

## Dependencies

- **serde**: Serialization/deserialization
- **clap**: Command-line argument parsing
- **rust_decimal**: Precise decimal arithmetic for financial calculations
- **tokio**: Async runtime for API calls
- **reqwest**: HTTP client for Yahoo Finance API
- **chrono**: Date and time handling

## Configuration

The `config.toml` file can be used to configure API endpoints and other settings.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This tool is for educational and research purposes only. Past performance does not guarantee future results. Always consult with financial professionals before making investment decisions.