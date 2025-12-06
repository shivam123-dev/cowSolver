use super::{Solver, SolverConfig, Solution, AuctionContext};
use crate::domain::{Order, OrderStatus};
use crate::settlement::SettlementPlan;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Main solver engine implementing batch auction logic
pub struct SolverEngine {
    config: SolverConfig,
    name: String,
}

impl SolverEngine {
    /// Creates a new solver engine with given configuration
    pub fn new(config: SolverConfig) -> Self {
        Self {
            config,
            name: "CoWSolverEngine".to_string(),
        }
    }

    /// Validates and filters orders before solving
    fn validate_orders(&self, orders: &[Order]) -> Vec<Order> {
        orders
            .iter()
            .filter(|order| {
                // Filter out invalid or expired orders
                if order.status != OrderStatus::Open {
                    debug!("Skipping non-open order: {:?}", order.id);
                    return false;
                }

                // Check if order is expired
                if let Some(valid_to) = order.valid_to {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as u32;
                    
                    if valid_to < now {
                        debug!("Skipping expired order: {:?}", order.id);
                        return false;
                    }
                }

                // Validate amounts are non-zero
                if order.sell_amount.is_zero() || order.buy_amount.is_zero() {
                    warn!("Skipping order with zero amounts: {:?}", order.id);
                    return false;
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Attempts to find CoW (Coincidence of Wants) matches
    async fn find_cow_matches(&self, orders: &[Order]) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();

        if !self.config.enable_cow_matching {
            return matches;
        }

        // Simple CoW matching: find orders that can be matched directly
        for (i, order_a) in orders.iter().enumerate() {
            for (j, order_b) in orders.iter().enumerate().skip(i + 1) {
                // Check if orders can be matched (sell token of A = buy token of B and vice versa)
                if order_a.sell_token == order_b.buy_token
                    && order_a.buy_token == order_b.sell_token
                {
                    // Check if price conditions are compatible
                    if self.is_price_compatible(order_a, order_b) {
                        debug!("Found CoW match: {:?} <-> {:?}", order_a.id, order_b.id);
                        matches.push((i, j));
                    }
                }
            }
        }

        info!("Found {} CoW matches", matches.len());
        matches
    }

    /// Checks if two orders have compatible prices for matching
    fn is_price_compatible(&self, order_a: &Order, order_b: &Order) -> bool {
        // Calculate limit prices
        // order_a wants: buy_amount / sell_amount
        // order_b wants: buy_amount / sell_amount
        
        // For a match to be valid:
        // order_a's limit price <= order_b's limit price (when normalized)
        
        // This is a simplified check - real implementation would use precise decimal math
        let price_a = order_a.buy_amount.as_u128() as f64 / order_a.sell_amount.as_u128() as f64;
        let price_b = order_b.sell_amount.as_u128() as f64 / order_b.buy_amount.as_u128() as f64;
        
        // Allow some tolerance for matching
        let tolerance = 1.0 + self.config.max_slippage / 100.0;
        price_a <= price_b * tolerance
    }

    /// Builds settlement plan from matched orders
    async fn build_settlement(
        &self,
        orders: &[Order],
        matches: Vec<(usize, usize)>,
    ) -> crate::Result<SettlementPlan> {
        let mut settlement = SettlementPlan::default();

        // For each match, create trades
        for (i, j) in matches {
            let order_a = &orders[i];
            let order_b = &orders[j];

            // Calculate clearing price (uniform price for both orders)
            // Use the geometric mean of the two limit prices
            let clearing_price = self.calculate_clearing_price(order_a, order_b);

            // Add clearing prices to settlement
            settlement.set_clearing_price(order_a.sell_token, clearing_price);
            settlement.set_clearing_price(order_a.buy_token, clearing_price);

            // Create trades for both orders
            // In a real implementation, this would calculate exact fill amounts
            settlement.add_trade(crate::settlement::Trade {
                order_id: order_a.id,
                executed_sell_amount: order_a.sell_amount,
                executed_buy_amount: order_a.buy_amount,
                fee: order_a.fee_amount,
            });

            settlement.add_trade(crate::settlement::Trade {
                order_id: order_b.id,
                executed_sell_amount: order_b.sell_amount,
                executed_buy_amount: order_b.buy_amount,
                fee: order_b.fee_amount,
            });
        }

        // If AMM routing is enabled, add AMM interactions for unmatched orders
        if self.config.enable_amm_routing {
            // TODO: Implement AMM routing logic
            debug!("AMM routing not yet implemented");
        }

        Ok(settlement)
    }

    /// Calculates uniform clearing price for matched orders
    fn calculate_clearing_price(&self, order_a: &Order, order_b: &Order) -> ethers::types::U256 {
        // Simplified clearing price calculation
        // Real implementation would use more sophisticated price discovery
        
        // Use geometric mean of the two limit prices
        let price_a = order_a.buy_amount.as_u128() as f64 / order_a.sell_amount.as_u128() as f64;
        let price_b = order_b.sell_amount.as_u128() as f64 / order_b.buy_amount.as_u128() as f64;
        
        let clearing_price = (price_a * price_b).sqrt();
        
        // Convert back to U256 (simplified)
        ethers::types::U256::from((clearing_price * 1e18) as u128)
    }

    /// Calculates total surplus generated by solution
    fn calculate_surplus(&self, orders: &[Order], settlement: &SettlementPlan) -> f64 {
        let mut total_surplus = 0.0;

        for trade in &settlement.trades {
            // Find corresponding order
            if let Some(order) = orders.iter().find(|o| o.id == trade.order_id) {
                // Surplus = (executed_buy_amount - expected_buy_amount)
                // This is simplified - real calculation would be more complex
                let executed = trade.executed_buy_amount.as_u128() as f64;
                let expected = order.buy_amount.as_u128() as f64;
                
                if executed > expected {
                    total_surplus += (executed - expected) / 1e18; // Convert from wei
                }
            }
        }

        total_surplus
    }
}

#[async_trait]
impl Solver for SolverEngine {
    async fn solve(&self, orders: Vec<Order>) -> crate::Result<Option<Solution>> {
        info!("Starting solver with {} orders", orders.len());

        // Validate and filter orders
        let valid_orders = self.validate_orders(&orders);
        
        if valid_orders.is_empty() {
            info!("No valid orders to solve");
            return Ok(None);
        }

        info!("Processing {} valid orders", valid_orders.len());

        // Find CoW matches
        let matches = self.find_cow_matches(&valid_orders).await;

        if matches.is_empty() {
            info!("No CoW matches found");
            // In a real implementation, we would try AMM routing here
            return Ok(None);
        }

        // Build settlement plan
        let settlement = self.build_settlement(&valid_orders, matches).await?;

        // Validate settlement
        settlement.validate()
            .map_err(|e| crate::Error::SettlementFailed(e))?;

        // Calculate gas cost
        let gas_cost = settlement.estimate_gas();

        // Calculate surplus
        let surplus = self.calculate_surplus(&valid_orders, &settlement);

        // Create solution
        let mut solution = Solution {
            orders: settlement.trades.iter().map(|t| t.order_id).collect(),
            settlement,
            gas_cost,
            surplus,
            score: 0.0,
        };

        // Calculate quality score
        solution.calculate_score();

        // Check if solution is profitable
        if !solution.is_profitable(self.config.min_profit_threshold) {
            warn!(
                "Solution not profitable: score={}, threshold={}",
                solution.score, self.config.min_profit_threshold
            );
            return Ok(None);
        }

        info!(
            "Found solution: {} orders, surplus={:.4}, score={:.4}",
            solution.orders.len(),
            solution.surplus,
            solution.score
        );

        Ok(Some(solution))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self) -> &SolverConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{OrderId, OrderKind};
    use ethers::types::{Address, U256};

    fn create_test_order(
        sell_token: Address,
        buy_token: Address,
        sell_amount: u128,
        buy_amount: u128,
    ) -> Order {
        Order {
            id: OrderId([0u8; 32]),
            owner: Address::zero(),
            sell_token,
            buy_token,
            sell_amount: U256::from(sell_amount),
            buy_amount: U256::from(buy_amount),
            valid_to: Some(u32::MAX),
            fee_amount: U256::from(1000),
            kind: OrderKind::Sell,
            partially_fillable: false,
            status: OrderStatus::Open,
            chain_id: crate::domain::ChainId::Mainnet,
        }
    }

    #[tokio::test]
    async fn test_solver_engine_creation() {
        let config = SolverConfig::default();
        let engine = SolverEngine::new(config);
        assert_eq!(engine.name(), "CoWSolverEngine");
    }

    #[tokio::test]
    async fn test_validate_orders() {
        let config = SolverConfig::default();
        let engine = SolverEngine::new(config);

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let orders = vec![
            create_test_order(token_a, token_b, 1000, 2000),
            create_test_order(token_a, token_b, 0, 2000), // Invalid: zero sell amount
        ];

        let valid = engine.validate_orders(&orders);
        assert_eq!(valid.len(), 1);
    }

    #[tokio::test]
    async fn test_cow_matching() {
        let config = SolverConfig::default();
        let engine = SolverEngine::new(config);

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let orders = vec![
            create_test_order(token_a, token_b, 1000, 2000),
            create_test_order(token_b, token_a, 2000, 1000),
        ];

        let matches = engine.find_cow_matches(&orders).await;
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (0, 1));
    }

    #[tokio::test]
    async fn test_solve_with_matches() {
        let config = SolverConfig::default();
        let engine = SolverEngine::new(config);

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let orders = vec![
            create_test_order(token_a, token_b, 1000000000000000000, 2000000000000000000),
            create_test_order(token_b, token_a, 2000000000000000000, 1000000000000000000),
        ];

        let solution = engine.solve(orders).await.unwrap();
        assert!(solution.is_some());

        let solution = solution.unwrap();
        assert_eq!(solution.orders.len(), 2);
        assert!(solution.score >= 0.0);
    }

    #[tokio::test]
    async fn test_solve_no_matches() {
        let config = SolverConfig::default();
        let engine = SolverEngine::new(config);

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);
        let token_c = Address::from_low_u64_be(3);

        let orders = vec![
            create_test_order(token_a, token_b, 1000, 2000),
            create_test_order(token_a, token_c, 1000, 3000),
        ];

        let solution = engine.solve(orders).await.unwrap();
        assert!(solution.is_none());
    }
}
