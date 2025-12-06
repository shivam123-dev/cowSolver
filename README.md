# CoW Protocol Cross-Chain Solver

High-performance cross-chain solver implementation for CoW Protocol, enabling seamless token swaps across multiple blockchain networks with optimal routing and settlement.

## Architecture

This solver implements a modular architecture based on CoW Protocol's design principles:

### Core Components

- **Order Book Service**: Manages incoming orders and batch construction
- **Solver Engine**: Optimizes settlement paths using CoW matching and AMM routing
- **Bridge Integration**: Handles cross-chain swaps via multiple bridge providers
- **Settlement Builder**: Constructs atomic on-chain transactions

### Cross-Chain Flow

1. User submits cross-chain swap order (Token X on Chain A → Token Y on Chain B)
2. Solver executes swap on source chain to intermediate token
3. Post-hook triggers bridge deposit within same transaction
4. Bridge provider completes transfer to destination chain
5. User receives target token on destination chain

## Project Structure

```
cowSolver/
├── crates/
│   ├── core/          # Pure solver logic, domain models
│   ├── adapters/      # Chain RPC, external integrations
│   ├── strategy/      # Solving strategies and optimization
│   └── bridge/        # Bridge provider integrations
├── bin/
│   ├── solver-cli/    # Command-line interface
│   └── solver-daemon/ # Long-running service
└── tests/             # Integration and e2e tests
```

## Features

- **CoW Matching**: Finds coincidence of wants across orders
- **Multi-AMM Routing**: Integrates Uniswap, Balancer, Curve
- **Cross-Chain Swaps**: Supports multiple bridge providers
- **Gas Optimization**: Minimizes transaction costs
- **Uniform Clearing Prices**: Ensures fair execution
- **Flash Loan Support**: Leverages flash loans for capital efficiency

## Building

```bash
cargo build --release
```

## Testing

```bash
# Unit tests
cargo test --workspace --lib

# Integration tests
cargo test --workspace --tests

# All tests
cargo test --workspace
```

## Configuration

See `config/` directory for environment-specific configurations.

## License

MIT
