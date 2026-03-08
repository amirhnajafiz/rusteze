# Rusteze

Fast and memory safe key-value storage in Rust.

## Features

- HTTP API
  - POST : Set [key, value, expire date]
  - GET : Get [key]
  - GET : Stats
- Prometheus metrics
- Jaeger tracer
- Zap logger
- Snapshot system (storing cache data from memory to disk)

## Configs

- HTTP port
- Metrics port
- Log level
- Snapshot enable
- Data directory
- Jaeger tracing
