# Stock DSL VS Code Extension

This extension provides syntax highlighting for Stock DSL files (`.stock` extension).

## Features

- Syntax highlighting for Stock DSL keywords: `INVEST`, `TIME`, `INVESTMENT`, `PATTERN`, `TEST`
- Support for time units: `10d`, `12w`, `2y`
- Comment highlighting for `//` style comments
- Number highlighting
- Stock ticker highlighting (2-5 letter uppercase symbols)

## Installation

1. Copy the `vscode-extension` folder to your VS Code extensions directory:
   - Windows: `%USERPROFILE%\.vscode\extensions\`
   - macOS: `~/.vscode/extensions/`
   - Linux: `~/.vscode/extensions/`

2. Restart VS Code

3. Open any `.stock` file to see syntax highlighting

## Usage

Create or open files with the `.stock` extension. The extension will automatically provide syntax highlighting based on the Stock DSL grammar.

Example `.stock` file:
```
// Test investment amounts
INVEST 10,100,1000

// Test time frames  
TIME 10d,12w,2y

INVESTMENT MSFT Microsoft
INVESTMENT AAPL Apple

PATTERN MyPattern Apple,Microsoft,Apple
TEST MyPattern
```