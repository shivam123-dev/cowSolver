pub mod engine;
pub mod matching;
pub mod routing;
pub mod pricing;

use crate::domain::{Order, OrderId};
use crate::settlement::SettlementPlan;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    /// Maximum gas price willing to pay (in gwei)
    pub max_gas_price: u64,
    
    /// Minimum profit threshold for solutions
    pub min_profit_threshold: f64,
    
    /// Maximum slippage tolerance (as percentage)
    pub max_slippage: f64,
    
    /// Enable CoW matching
    pub enable_cow_matching: bool,
    
    /// Enable AMM routing
    pub enable_amm_routing: bool,
    
    /// Enable cross-chain swaps
    pub enable_cross_chain: bool,
    
    /// Solver timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_gas_price: 100,
            min_profit_threshold: 0.01,
            max_slippage: 0.5,
            enable_cow_matching: true,
            enable_amm_routing: true,
            enable_cross_chain: true,
            timeout_ms: 5000,
        }
    }
}

/// Solution produced by solver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    /// Orders included in solution
    pub orders: Vec<OrderId>,
    
    /// Settlement plan
    pub settlement: SettlementPlan,
    
    /// Estimated gas cost
    pub gas_cost: u64,
    
    /// Total surplus generated
    pub surplus: f64,
    
    /// Solution quality score
    pub score: f64,
}

/// Solver trait for different solving strategies
#[async_trait]
pub trait Solver: Send + Sync {
    /// Solves a batch of orders
    async fn solve(&self, orders: Vec<Order>) -> crate::Result<Option<Solution>>;
    
    /// Returns solver name
    fn name(&self) -> &str;
    
    /// Returns solver configuration
    fn config(&self) -> &SolverConfig;
}

/// Batch auction context
#[derive(Debug, Clone)]
pub struct AuctionContext {
    /// Current block number
    pub block_number: u64,
    
    /// Current timestamp
    pub timestamp: u32,
    
    /// Current gas price
    pub gas_price: u64,
    
    /// Available liquidity sources
    pub liquidity_sources: Vec<String>,
}

impl Solution {
    /// Calculates solution quality score
    pub fn calculate_score(&mut self) {
        // Score = surplus - gas_cost_in_eth
        // Higher surplus and lower gas cost = better score
        let gas_cost_eth = self.gas_cost as f64 * 1e-9; // Convert gwei to ETH
        self.score = self.surplus - gas_cost_eth;
    }
    
    /// Checks if solution is profitable
    pub fn is_profitable(&self, min_threshold: f64) -> bool {
        self.score >= min_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = SolverConfig::default();
        assert_eq!(config.max_gas_price, 100);
        assert!(config.enable_cow_matching);
    }
    
    #[test]
    fn test_solution_scoring() {
        let mut solution = Solution {
            orders: vec![],
            settlement: SettlementPlan::default(),
            gas_cost: 100_000,
            surplus: 0.5,
            score: 0.0,
        };
        
        solution.calculate_score();
        assert!(solution.score > 0.0);
        assert!(solution.is_profitable(0.0));
    }
}
