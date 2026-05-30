# NHL Schedule CLI

A Rust command-line application that displays the next 7 days of NHL games using public ESPN API data.

## Features

- Automatically displays upcoming NHL games for the next 7 days
- Plain text output format (no ASCII boxes or tables)
- Shows games with full team names (no abbreviations)
- Displays timezone offsets (e.g., UTC-4) with all times
- Numeric date format (YYYY-MM-DD) for date headers
- Includes venue information when available
- Graceful error handling for API failures

## Requirements

- Rust (latest stable version)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/kklopfenstein/hockey-info.git
cd hockey-info
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

Or install the binary:
```bash
cargo install --path .
```

## Usage

The application runs automatically and displays the next 7 days of NHL games without requiring any command-line arguments.

**Output format example:**
```
2026-05-30:
Toronto Maple Leafs @ Montreal Canadiens
    Scheduled | 19:00 (UTC-4 America/New_York)

2026-06-01:
Boston Bruins @ New York Rangers
    Scheduled | 20:00 (UTC-4 America/New_York)
```

## API

Uses the public ESPN NHL scoreboard API:
- Endpoint: `https://site.api.espn.com/apis/site/v2/sports/hockey/nhl/scoreboard`
- No authentication required
- Returns game schedule with team names, times, and venue information

## Project Structure

```
hockey-info/
├── Cargo.toml          # Project dependencies
├── src/
│   └── main.rs         # Main application logic
└── README.md           # This file
```

## Technologies

- **Rust** - Programming language
- **reqwest** - HTTP client for API requests
- **serde/serde_json** - JSON serialization/deserialization
- **chrono** - Date and time handling with timezone support
- **tokio** - Async runtime

## License

MIT License