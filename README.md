# Stock Simulator

A simple command-line stock market simulator written in Rust that models investment growth over time using configurable weekly gains.

## Features

- Simulate stock investments over a specified number of weeks
- Support for multiple stocks with different gain rates
- Configurable through TOML configuration files
- Precise decimal calculations using `rust_decimal`
- Command-line interface with optional config file path

## Installation

Make sure you have Rust installed on your system. Then build the project:

```bash
cargo build --release
```

## Usage

### Basic Usage

Run the simulator with the default `config.toml` file:

```bash
cargo run
```

### Using a Custom Configuration File

```bash
cargo run -- --config path/to/your/config.toml
```

Or use the short flag:

```bash
cargo run -- -c path/to/your/config.toml
```

## Configuration

The simulator uses TOML configuration files. Here's the structure:

```toml
initial_amount = 100.0    # Starting investment amount
weeks = 52               # Number of weeks to simulate
gains = [0.09, 0.50]     # Weekly gains for each stock (cycles through)
```

### Configuration Parameters

- `initial_amount`: The starting investment amount (decimal)
- `weeks`: Number of weeks to simulate the investment
- `gains`: Array of weekly gain amounts for different stocks. The simulator cycles through these values.

## Example

With the default configuration:
- Initial investment: $100.00
- Simulation period: 52 weeks
- Stock gains: $0.09 and $0.50 (alternating weekly)

The simulator will show:
- Final total amount
- Total amount gained
- Percentage gain

## Output

```
Simulating 52 weeks with an initial amount of 100 from "config.toml"...

--- Results ---
Final Total Amount: 115.34
Total Amount Gained: 15.34
Percentage Gain: 15.34%
```

## License

This project is licensed under the terms specified in the LICENSE file.

## Dependencies

- `clap`: Command-line argument parsing
- `rust_decimal`: Precise decimal arithmetic
- `serde`: Serialization/deserialization
- `toml`: TOML configuration file parsing