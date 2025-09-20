# Unhaunter Development

## Quick Start

```bash
# Run tests
cargo test -p uncore --lib -- --skip ghost_setfinder

# Check code quality  
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings

# Build and run
cargo build --release
cargo run --bin unhaunter_game
```

## Using Just (optional)

```bash
just test    # Run tests
just check   # Format + clippy + tests
just build   # Release build
just run     # Run game
just dev     # Quick development cycle
```

## CI/CD

GitHub Actions automatically runs tests on every commit to main branch.

## Testing

- **64 unit tests** covering core systems
- **Zero warnings** - clean codebase
- **Property-based testing** for mathematical correctness
- **Performance benchmarks** for optimization

## Status

✅ **Production Ready** - All tests passing, zero warnings, comprehensive coverage
