pub mod domain;
pub mod solver;
pub mod settlement;
pub mod math;

pub use solver::{Solver, SolverConfig, Solution};
pub use domain::{Order, Token, ChainId, OrderStatus};
pub use settlement::{Settlement, SettlementPlan};

/// Core result type for solver operations
pub type Result<T> = std::result::Result<T, Error>;

/// Core error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid order: {0}")]
    InvalidOrder(String),
    
    #[error("Insufficient liquidity: {0}")]
    InsufficientLiquidity(String),
    
    #[error("Settlement failed: {0}")]
    SettlementFailed(String),
    
    #[error("Bridge error: {0}")]
    BridgeError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
