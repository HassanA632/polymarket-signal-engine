# Polymarket Signal Engine

A Rust-based low-latency Polymarket market-data and signal engine.

I'm building this project as a time-critical market data system for prediction markets. The goal is to ingest live Polymarket data, maintain local market state, and eventually generate real-time trading signals for research and paper trading strategies.

## Project Goals

- Fetch active Polymarket events and markets
- Identify tradable markets with useful liquidity and volume
- Stream live market and order-book data
- Maintain local in-memory market state
- Generate real-time signals from market movements
- Measure processing latency across the data pipeline
- Support paper-trading strategy research
- Keep the system modular, testable, and suitable for low-latency development

## Why This Project?

Prediction markets are event-driven and time-sensitive. Market prices can change quickly as new information appears, so a useful system needs to process incoming data efficiently and react with minimal delay.

This project will include:

- asynchronous Rust
- WebSocket data ingestion
- concurrent processing
- bounded message channels
- in-memory state management
- latency-aware design
- structured logging and observability
- clean separation between data ingestion, state updates, and signal generation

## Status

This project is currently in active development.

Implemented:

- Basic CLI structure
- Polymarket event fetching
- Market search and display filtering
- Market inspection by market ID
- Market inspection by CLOB token ID
- Live WebSocket market stream
- Typed WebSocket message parsing
- Live token market state tracking
- Previous-state comparisons
- Processing latency metrics
- p50, p95, and p99 latency statistics
- Signal engine
- Configurable signal thresholds
- Text and JSON signal output
- Optional JSONL signal logging
- Stream output controls
- Signal metrics
- Unit tests for core parsing, filtering, state, metrics, and signal logic

## Planned Features

### Market Discovery

- Filter inactive child markets
- Display CLOB token IDs for YES/NO outcomes
- Add sorting options for volume, liquidity, and end date
- Add CLI flags for market limits and filters

### Live Market Data

- Connect to Polymarket WebSocket streams
- Subscribe to selected markets
- Parse live order-book and trade updates
- Maintain local best bid, best ask, spread, and mid-price

### Low-Latency Engine

- Use Tokio tasks for concurrent data ingestion and processing
- Use bounded channels to handle backpressure
- Avoid blocking operations on the hot path
- Track message processing latency
- Report p50, p95, and p99 latency metrics

### Signal Engine

- Detect price movement
- Detect spread changes
- Detect order-book imbalance
- Detect volume spikes
- Emit structured signal events

### Paper Trading

- Add experimental paper-trading mode
- Open simulated positions from selected signal conditions
- Track simulated entry price, position state, and strategy reason
- Add configurable stake size
- Add simple take-profit and stop-loss exits
- Track realised and unrealised paper PnL
- Print paper-trading session summaries
- Optionally log paper trades as JSONL

## Architecture Direction

The intended architecture is:

```text
CLI
 ↓
Polymarket API client
 ↓
Market discovery
 ↓
WebSocket ingestion
 ↓
Local order-book state
 ↓
Signal engine
 ↓
Paper trading / metrics / output
```

The future live-data pipeline will be designed around a low-latency hot path:

```text
WebSocket message received
        ↓
Parse update
        ↓
Update local market state
        ↓
Calculate signal
        ↓
Emit event
```

Slow operations such as database writes, verbose terminal output, and historical logging should not block the hot path.

## Tech Stack

- Rust
- Tokio
- Reqwest
- Serde
- Serde JSON
- Clap
- Tracing
- Tokio Tungstenite
- Futures Util

Planned additions:

- Criterion benchmarks
- SQLx or SQLite/Postgres for historical storage
- More advanced metrics collection for latency tracking

## Running the Project

Fetch active Polymarket events:

```bash
cargo run -- markets
```

Fetch a limited number of events:

```bash
cargo run -- markets --limit 5
```

Limit how many tradable markets are displayed per event:

```bash
cargo run -- markets --limit 5 --max-display-markets 3
```

Search fetched events and markets by keyword:

```bash
cargo run -- markets --search bitcoin --limit 100 --max-display-markets 5
```

Inspect a market by market ID:

```bash
cargo run -- inspect --market-id <MARKET_ID> --limit 100
```

Inspect a market by CLOB token ID:

```bash
cargo run -- inspect --token-id <CLOB_TOKEN_ID> --limit 100
```

Stream live market data for a CLOB token ID:

```bash
cargo run -- stream --token-id <CLOB_TOKEN_ID>
```

Stream with state summaries enabled:

```bash
cargo run -- stream --token-id <CLOB_TOKEN_ID> --show-state
```

Stream with parsed WebSocket event summaries enabled:

```bash
cargo run -- stream --token-id <CLOB_TOKEN_ID> --show-events
```

Use JSON signal output:

```bash
cargo run -- stream --token-id <CLOB_TOKEN_ID> --output json
```

Log emitted signals as JSONL:

```bash
cargo run -- stream --token-id <CLOB_TOKEN_ID> --output json --log-signals signals.jsonl
```

Tune signal thresholds:

```bash
cargo run -- stream --token-id <CLOB_TOKEN_ID> \
  --tight-spread-threshold 0.02 \
  --min-spread-tightening 0.005 \
  --min-price-move 0.01 \
  --large-trade-threshold 250
```

## Development Principles

- Correctness before optimisation
- Low-latency architecture from the start
- Minimal blocking work on the hot path
- Clear module boundaries
- Strong typing for external API responses
- Structured logging instead of scattered print statements
