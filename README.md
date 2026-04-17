# Ria Coder

**Terminal-based agentic coding system written in Rust**

Ria Coder brings autonomous AI coding capabilities directly to your terminal. Built on the RIA inference platform with 100% local inference.

## Quick Start

```bash
# Build
cargo build --release

# Start interactive session
cargo run -- coder

# Generate code
cargo run -- generate --prompt "Write a Fibonacci function in Rust"

# Inspect model
cargo run -- inspect --model path/to/model.gguf
```

## Project Structure

```
ria-coder/
├── crates/
│   ├── tui/          # Terminal UI (ratatui-based)
│   ├── agent/        # Agent orchestration
│   ├── tools/        # Tool integration layer
│   ├── config/       # Configuration management
│   └── cli/          # CLI binary (ria)
├── assets/
│   └── themes/       # Color themes
├── examples/         # Usage examples
└── tests/            # Integration tests
```

## Features

- 🖥️ **Terminal-Native**: Full TUI with ratatui
- 🤖 **Agentic**: Autonomous multi-step task execution
- 🔒 **100% Local**: No data leaves your machine
- 🛡️ **Safe by Default**: All changes reviewed before application
- ⚡ **Rust-Fast**: Single binary, zero dependencies

## Specifications

Full specification in [../ria-coder-spec/](../ria-coder-spec/):
- 41 specification documents
- TUI, Agent, Tools, Safety, Performance
- Local-First Architecture

## Dependencies

- **ria-platform**: GGUF parser + inference engine
- **ratatui**: Terminal UI framework
- **tokio**: Async runtime
- **gix**: Git integration

## License

DOSL-IIE-1.0 (Dust Open Source License - Intelligence Infrastructure Edition)
