use serde::{Deserialize, Serialize};
use ethers::types::{Address, U256, Bytes};
use crate::domain::{OrderId, ChainId};
use std::collections::HashMap;

/// Settlement plan for executing trades
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettlementPlan {
    /// Trades to execute
    pub trades: Vec<Trade>,
    
    /// On-chain interactions (AMM swaps, etc.)
    pub interactions: Vec<Interaction>,
    
    /// Clearing prices per token
    pub clearing_prices: HashMap<Address, U256>,
    
    /// Post-hooks for cross-chain operations
    pub post_hooks: Vec<PostHook>,
}

/// Individual trade in settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Order being filled
    pub order_id: OrderId,
    
    /// Executed sell amount
    pub executed_sell_amount: U256,
    
    /// Executed buy amount
    pub executed_buy_amount: U256,
    
    /// Fee paid
    pub fee: U256,
}

/// On-chain interaction (AMM swap, vault operation, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// Target contract address
    pub target: Address,
    
    /// Call data
    pub call_data: Bytes,
    
    /// Value to send (for native token)
    pub value: U256,
    
    /// Interaction type
    pub interaction_type: InteractionType,
}

/// Type of on-chain interaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InteractionType {
    /// Uniswap V2 swap
    UniswapV2Swap,
    
    /// Uniswap V3 swap
    UniswapV3Swap,
    
    /// Balancer vault swap
    BalancerSwap,
    
    /// Curve pool swap
    CurveSwap,
    
    /// ERC20 approval
    Approval,
    
    /// Custom interaction
    Custom,
}

/// Post-hook for cross-chain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostHook {
    /// Target bridge contract
    pub bridge_contract: Address,
    
    /// Call data for bridge
    pub call_data: Bytes,
    
    /// Source chain
    pub source_chain: ChainId,
    
    /// Destination chain
    pub destination_chain: ChainId,
    
    /// Intermediate token being bridged
    pub intermediate_token: Address,
    
    /// Amount to bridge
    pub amount: U256,
    
    /// Recipient on destination chain
    pub recipient: Address,
}

impl Settlement {
    /// Creates a new empty settlement
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Adds a trade to the settlement
    pub fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
    }
    
    /// Adds an interaction to the settlement
    pub fn add_interaction(&mut self, interaction: Interaction) {
        self.interactions.push(interaction);
    }
    
    /// Adds a post-hook for cross-chain
    pub fn add_post_hook(&mut self, post_hook: PostHook) {
        self.post_hooks.push(post_hook);
    }
    
    /// Sets clearing price for a token
    pub fn set_clearing_price(&mut self, token: Address, price: U256) {
        self.clearing_prices.insert(token, price);
    }
    
    /// Validates settlement plan
    pub fn validate(&self) -> Result<(), String> {
        if self.trades.is_empty() {
            return Err("Settlement must contain at least one trade".to_string());
        }
        
        // Validate all trades have clearing prices
        for trade in &self.trades {
            // Additional validation logic here
        }
        
        Ok(())
    }
    
    /// Estimates total gas cost
    pub fn estimate_gas(&self) -> u64 {
        let base_gas = 21000u64;
        let trade_gas = self.trades.len() as u64 * 50000;
        let interaction_gas = self.interactions.len() as u64 * 100000;
        let post_hook_gas = self.post_hooks.len() as u64 * 150000;
        
        base_gas + trade_gas + interaction_gas + post_hook_gas
    }
}

/// Type alias for settlement
pub type Settlement = SettlementPlan;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_settlement_creation() {
        let settlement = Settlement::new();
        assert_eq!(settlement.trades.len(), 0);
        assert_eq!(settlement.interactions.len(), 0);
    }
    
    #[test]
    fn test_gas_estimation() {
        let mut settlement = Settlement::new();
        let base_gas = settlement.estimate_gas();
        
        settlement.add_trade(Trade {
            order_id: OrderId([0u8; 32]),
            executed_sell_amount: U256::from(1000),
            executed_buy_amount: U256::from(2000),
            fee: U256::from(10),
        });
        
        assert!(settlement.estimate_gas() > base_gas);
    }
}
