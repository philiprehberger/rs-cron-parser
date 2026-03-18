# rs-cron-parser

[![CI](https://github.com/philiprehberger/rs-cron-parser/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-cron-parser/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-cron-parser.svg)](https://crates.io/crates/philiprehberger-cron-parser)
[![License](https://img.shields.io/github/license/philiprehberger/rs-cron-parser)](LICENSE)

Cron expression parsing, scheduling, and human-readable descriptions

## Installation

```toml
[dependencies]
philiprehberger-cron-parser = "0.1.6"
```

## Usage

```rust
use philiprehberger_cron_parser::{CronExpr, DateTime};

// Parse a cron expression
let expr = CronExpr::parse("0 9 * * 1-5").unwrap();

// Check next execution from a given time
let now = DateTime { year: 2026, month: 3, day: 15, hour: 8, minute: 0, second: 0 };
let next = expr.next_from(&now).unwrap();
// next = DateTime { year: 2026, month: 3, day: 16, hour: 9, minute: 0, second: 0 }

// Human-readable description
println!("{}", expr.describe());
// "At 9:00 AM, Monday through Friday"

// Use aliases
let hourly = CronExpr::parse("@hourly").unwrap();
```

## API

| Function / Type | Description |
|-----------------|-------------|
| `CronExpr::parse(expr)` | Parse a cron expression |
| `.next_from(dt)` | Next execution time after dt |
| `.next_n_from(dt, n)` | Next N execution times |
| `.matches(dt)` | Check if dt matches the expression |
| `.describe()` | Human-readable description |
| `DateTime` | Simple date/time struct |


## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## License

MIT
