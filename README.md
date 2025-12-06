# CoW Protocol Cross-Chain Solver

High-performance cross-chain solver implementation for CoW Protocol, enabling seamless token swaps across multiple blockchain networks with optimal routing and settlement.

## 🚀 Implementation Status

### ✅ Completed (Core Solver)
- **Solver Engine** - Batch auction processing and orchestration
- **Order Matching** - CoW discovery with direct pair and ring matching
- **AMM Routing** - Multi-hop routing through Uniswap, Balancer, Curve
- **Pricing Engine** - Uniform clearing price calculation with multiple strategies
- **Domain Models** - Orders, tokens, chains, settlement structures
- **Math Utilities** - Pricing calculations and decimal handling

### 🔄 In Progress
- **Adapters Layer** - Chain RPC clients and DEX integrations
- **Bridge Integration** - Cross-chain settlement execution
- **Strategy Layer** - Advanced solving strategies

### 📋 Planned
- **CLI Binary** - Command-line interface for solver operations
- **Daemon Service** - Long-running solver service with API
- **Integration Tests** - End-to-end testing suite
- **Performance Optimization** - Gas optimization and parallel processing

See [Development Log](docs/DEVELOPMENT_LOG.md) for detailed progress tracking.

---

## Architecture

This solver implements a modular architecture based on CoW Protocol's design principles:

### Core Components

- **Solver Engine** (`crates/core/src/solver/engine.rs`)
  - Batch auction processing
  - Order validation and filtering
  - Solution quality scoring
  - Profitability checks

- **Matching Engine** (`crates/core/src/solver/matching.rs`)
  - Direct pair matching (A ↔ B)
  - Ring matching (A → B → C → A)
  - Price overlap detection
  - Match quality scoring

- **Routing Engine** (`crates/core/src/solver/routing.rs`)
  - Multi-hop AMM routing
  - Liquidity pool modeling
  - Price impact estimation
  - Gas-aware route selection

- **Pricing Engine** (`crates/core/src/solver/pricing.rs`)
  - Uniform clearing prices
  - Multiple pricing strategies
  - Surplus calculation
  - Fee optimization

- **Settlement Builder** (`crates/core/src/settlement/mod.rs`)
  - Trade execution plans
  - On-chain interactions
  - Cross-chain post-hooks

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
│   ├── core/              # ✅ Pure solver logic, domain models
│   │   ├── domain/        # ✅ Order, token, chain models
│   │   ├── solver/        # ✅ Solver engine and algorithms
│   │   │   ├── engine.rs  # ✅ Main solver orchestration
│   │   │   ├── matching.rs # ✅ CoW matching algorithms
│   │   │   ├── routing.rs  # ✅ AMM routing engine
│   │   │   └── pricing.rs  # ✅ Pricing strategies
│   │   ├── settlement/    # ✅ Settlement plan structures
│   │   └── math/          # ✅ Mathematical utilities
│   ├── adapters/          # 🔄 Chain RPC, external integrations
│   ├── strategy/          # 🔄 Solving strategies and optimization
│   └── bridge/            # 🔄 Bridge provider integrations
├── bin/
│   ├── solver-cli/        # 📋 Command-line interface
│   └── solver-daemon/     # 📋 Long-running service
├── docs/
│   └── DEVELOPMENT_LOG.md # 📊 Implementation progress tracking
└── tests/                 # 📋 Integration and e2e tests
```

**Legend:** ✅ Complete | 🔄 In Progress | 📋 Planned

## Features

### Implemented ✅
- **CoW Matching**: Direct pair and ring matching with quality scoring
- **AMM Routing**: Multi-hop routing through multiple DEX protocols
- **Pricing Strategies**: MidPoint, MaxSurplus, MarketPrice, VolumeWeighted
- **Gas Optimization**: Gas-aware route selection and cost estimation
- **Uniform Clearing Prices**: Fair execution with surplus maximization
- **Order Validation**: Comprehensive validation and filtering
- **Price Impact**: Real-time price impact calculation
- **Quality Scoring**: Solution ranking and selection

### Planned 📋
- **Multi-AMM Integration**: Full Uniswap V2/V3, Balancer, Curve support
- **Cross-Chain Swaps**: Bridge provider integrations
- **Flash Loan Support**: Capital efficiency optimization
- **MEV Protection**: Front-running and sandwich attack prevention
- **Real-time Monitoring**: Metrics and observability

## Getting Started

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/0xtechroot/cowSolver.git
cd cowSolver

# Build all crates
cargo build --release

# Build specific crate
cargo build -p core --release
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific module
cargo test -p core --lib

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_solver_engine_creation
```

### Development

```bash
# Check code
cargo check --workspace

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace -- -D warnings

# Generate documentation
cargo doc --workspace --no-deps --open
```

## Configuration

Solver configuration is managed through `SolverConfig`:

```rust
use cowsolver::solver::SolverConfig;

let config = SolverConfig {
    max_gas_price: 100,           // Max gas price in gwei
    min_profit_threshold: 0.01,   // Minimum profit (1%)
    max_slippage: 0.5,            // Max slippage (0.5%)
    enable_cow_matching: true,    // Enable CoW matching
    enable_amm_routing: true,     // Enable AMM routing
    enable_cross_chain: true,     // Enable cross-chain
    timeout_ms: 5000,             // Solver timeout
};
```

## Usage Example

```rust
use cowsolver::solver::{SolverEngine, SolverConfig};
use cowsolver::domain::Order;

#[tokio::main]
async fn main() {
    // Create solver with default config
    let config = SolverConfig::default();
    let solver = SolverEngine::new(config);
    
    // Prepare orders
    let orders: Vec<Order> = vec![/* ... */];
    
    // Solve batch
    match solver.solve(orders).await {
        Ok(Some(solution)) => {
            println!("Found solution!");
            println!("Orders: {}", solution.orders.len());
            println!("Surplus: {:.4}", solution.surplus);
            println!("Score: {:.4}", solution.score);
        }
        Ok(None) => println!("No solution found"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines and branching strategy.

## Documentation

- [Development Log](docs/DEVELOPMENT_LOG.md) - Implementation progress
- [Contributing Guide](CONTRIBUTING.md) - Development guidelines
- [API Documentation](https://docs.rs/cowsolver) - Generated docs (coming soon)

## Performance

Current benchmarks (on development machine):
- Order validation: ~1μs per order
- CoW matching: ~10μs per order pair
- Route finding: ~100μs per route
- Solution scoring: ~5μs per solution

*Note: Benchmarks are preliminary and will be formalized in future releases.*

## Roadmap

### Phase 1: Core Solver ✅ (Current)
- [x] Domain models
- [x] Solver engine
- [x] Order matching
- [x] AMM routing
- [x] Pricing strategies

### Phase 2: Integration 🔄 (Next)
- [ ] Chain RPC adapters
- [ ] DEX protocol integrations
- [ ] Price oracle connections
- [ ] Bridge integrations

### Phase 3: Production 📋
- [ ] CLI and daemon binaries
- [ ] Monitoring and metrics
- [ ] Performance optimization
- [ ] Security audits

### Phase 4: Advanced Features 📋
- [ ] Flash loan integration
- [ ] MEV protection
- [ ] Advanced routing strategies
- [ ] Multi-chain support

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- CoW Protocol team for the batch auction design
- Ethereum DeFi community for AMM innovations
- Rust community for excellent tooling

---

**Status:** Active Development  
**Version:** 0.1.0 (Pre-release)  
**Last Updated:** December 6, 2025

For questions or support, please open an issue on GitHub.
