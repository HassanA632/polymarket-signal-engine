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

- Simulate entries and exits
- Track profit and loss
- Include simple slippage assumptions
- Record strategy performance over time

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
- Clap
- Tracing

Planned additions:

- WebSocket client
- Criterion benchmarks
- SQLx or SQLite/Postgres for historical storage
- Metrics collection for latency tracking

## Running the Project

Fetch active Polymarket events:

```bash
cargo run -- markets
```

Fetch a limited number of events:

```bash
cargo run -- markets --limit 5
```

or:

```bash
cargo run -- markets -l 5
```

## Development Principles

- Correctness before optimisation
- Low-latency architecture from the start
- Minimal blocking work on the hot path
- Clear module boundaries
- Strong typing for external API responses
- Structured logging instead of scattered print statements
