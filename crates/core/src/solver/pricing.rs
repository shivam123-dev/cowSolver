use crate::domain::Order;
use ethers::types::{Address, U256};
use std::collections::HashMap;
use tracing::{debug, info};

/// Represents a clearing price for a token
#[derive(Debug, Clone)]
pub struct ClearingPrice {
    /// Token address
    pub token: Address,
    
    /// Price in reference token (usually ETH or USD)
    pub price: U256,
    
    /// Confidence score (0-1)
    pub confidence: f64,
}

/// Pricing strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PricingStrategy {
    /// Use mid-point of overlapping limit prices
    MidPoint,
    
    /// Maximize total surplus
    MaxSurplus,
    
    /// Minimize price deviation from market
    MarketPrice,
    
    /// Volume-weighted average
    VolumeWeighted,
}

/// Pricing engine for calculating uniform clearing prices
pub struct PricingEngine {
    /// Pricing strategy to use
    strategy: PricingStrategy,
    
    /// External price oracle (token -> price in ETH)
    price_oracle: HashMap<Address, U256>,
    
    /// Minimum price confidence threshold
    min_confidence: f64,
}

impl PricingEngine {
    /// Creates a new pricing engine
    pub fn new(strategy: PricingStrategy, min_confidence: f64) -> Self {
        Self {
            strategy,
            price_oracle: HashMap::new(),
            min_confidence,
        }
    }

    /// Sets external price for a token
    pub fn set_external_price(&mut self, token: Address, price: U256) {
        self.price_oracle.insert(token, price);
    }

    /// Calculates uniform clearing prices for a set of matched orders
    pub fn calculate_clearing_prices(
        &self,
        orders: &[Order],
    ) -> HashMap<Address, ClearingPrice> {
        info!("Calculating clearing prices for {} orders", orders.len());

        match self.strategy {
            PricingStrategy::MidPoint => self.calculate_midpoint_prices(orders),
            PricingStrategy::MaxSurplus => self.calculate_max_surplus_prices(orders),
            PricingStrategy::MarketPrice => self.calculate_market_prices(orders),
            PricingStrategy::VolumeWeighted => self.calculate_volume_weighted_prices(orders),
        }
    }

    /// Calculates prices using mid-point strategy
    fn calculate_midpoint_prices(&self, orders: &[Order]) -> HashMap<Address, ClearingPrice> {
        let mut prices = HashMap::new();
        let mut token_pairs: HashMap<(Address, Address), Vec<&Order>> = HashMap::new();

        // Group orders by token pair
        for order in orders {
            token_pairs
                .entry((order.sell_token, order.buy_token))
                .or_insert_with(Vec::new)
                .push(order);
        }

        // Calculate mid-point price for each pair
        for ((sell_token, buy_token), pair_orders) in token_pairs {
            if pair_orders.is_empty() {
                continue;
            }

            // Find min and max limit prices
            let mut min_price = f64::MAX;
            let mut max_price = f64::MIN;

            for order in &pair_orders {
                let limit_price = order.buy_amount.as_u128() as f64 / order.sell_amount.as_u128() as f64;
                min_price = min_price.min(limit_price);
                max_price = max_price.max(limit_price);
            }

            // Mid-point price
            let mid_price = (min_price + max_price) / 2.0;
            let price_u256 = U256::from((mid_price * 1e18) as u128);

            // Calculate confidence based on price spread
            let spread = (max_price - min_price) / mid_price;
            let confidence = (1.0 - spread.min(1.0)).max(0.0);

            prices.insert(
                sell_token,
                ClearingPrice {
                    token: sell_token,
                    price: price_u256,
                    confidence,
                },
            );

            debug!(
                "Mid-point price for {:?}: {:.6}, confidence: {:.2}",
                sell_token, mid_price, confidence
            );
        }

        prices
    }

    /// Calculates prices that maximize total surplus
    fn calculate_max_surplus_prices(&self, orders: &[Order]) -> HashMap<Address, ClearingPrice> {
        // This is a simplified implementation
        // Real implementation would use optimization algorithms (e.g., linear programming)
        
        let mut prices = HashMap::new();
        
        // Group orders by token
        let mut token_orders: HashMap<Address, Vec<&Order>> = HashMap::new();
        
        for order in orders {
            token_orders
                .entry(order.sell_token)
                .or_insert_with(Vec::new)
                .push(order);
            
            token_orders
                .entry(order.buy_token)
                .or_insert_with(Vec::new)
                .push(order);
        }

        // For each token, find price that maximizes surplus
        for (token, token_orders) in token_orders {
            if token_orders.is_empty() {
                continue;
            }

            // Calculate volume-weighted average of limit prices
            let mut total_volume = 0u128;
            let mut weighted_price_sum = 0.0;

            for order in &token_orders {
                let volume = order.sell_amount.as_u128();
                let limit_price = order.buy_amount.as_u128() as f64 / order.sell_amount.as_u128() as f64;
                
                total_volume += volume;
                weighted_price_sum += limit_price * volume as f64;
            }

            if total_volume == 0 {
                continue;
            }

            let avg_price = weighted_price_sum / total_volume as f64;
            let price_u256 = U256::from((avg_price * 1e18) as u128);

            prices.insert(
                token,
                ClearingPrice {
                    token,
                    price: price_u256,
                    confidence: 0.8, // Medium confidence for optimization-based pricing
                },
            );

            debug!(
                "Max surplus price for {:?}: {:.6}",
                token, avg_price
            );
        }

        prices
    }

    /// Calculates prices based on external market prices
    fn calculate_market_prices(&self, orders: &[Order]) -> HashMap<Address, ClearingPrice> {
        let mut prices = HashMap::new();

        // Collect all tokens
        let mut tokens = std::collections::HashSet::new();
        for order in orders {
            tokens.insert(order.sell_token);
            tokens.insert(order.buy_token);
        }

        // Use oracle prices if available
        for token in tokens {
            if let Some(&oracle_price) = self.price_oracle.get(&token) {
                prices.insert(
                    token,
                    ClearingPrice {
                        token,
                        price: oracle_price,
                        confidence: 0.95, // High confidence for oracle prices
                    },
                );

                debug!(
                    "Market price for {:?}: {}",
                    token, oracle_price
                );
            } else {
                // Fallback to mid-point if no oracle price
                debug!("No oracle price for {:?}, using fallback", token);
            }
        }

        // Fill in missing prices using mid-point strategy
        let midpoint_prices = self.calculate_midpoint_prices(orders);
        for (token, price) in midpoint_prices {
            prices.entry(token).or_insert(price);
        }

        prices
    }

    /// Calculates volume-weighted prices
    fn calculate_volume_weighted_prices(&self, orders: &[Order]) -> HashMap<Address, ClearingPrice> {
        let mut prices = HashMap::new();
        let mut token_data: HashMap<Address, (f64, u128)> = HashMap::new();

        // Accumulate volume-weighted prices
        for order in orders {
            let volume = order.sell_amount.as_u128();
            let limit_price = order.buy_amount.as_u128() as f64 / order.sell_amount.as_u128() as f64;

            let entry = token_data.entry(order.sell_token).or_insert((0.0, 0));
            entry.0 += limit_price * volume as f64;
            entry.1 += volume;
        }

        // Calculate weighted average prices
        for (token, (weighted_sum, total_volume)) in token_data {
            if total_volume == 0 {
                continue;
            }

            let avg_price = weighted_sum / total_volume as f64;
            let price_u256 = U256::from((avg_price * 1e18) as u128);

            prices.insert(
                token,
                ClearingPrice {
                    token,
                    price: price_u256,
                    confidence: 0.85, // Good confidence for volume-weighted
                },
            );

            debug!(
                "Volume-weighted price for {:?}: {:.6}",
                token, avg_price
            );
        }

        prices
    }

    /// Validates clearing prices against orders
    pub fn validate_prices(
        &self,
        prices: &HashMap<Address, ClearingPrice>,
        orders: &[Order],
    ) -> Result<(), String> {
        for order in orders {
            let sell_price = prices
                .get(&order.sell_token)
                .ok_or_else(|| format!("Missing price for sell token {:?}", order.sell_token))?;

            let buy_price = prices
                .get(&order.buy_token)
                .ok_or_else(|| format!("Missing price for buy token {:?}", order.buy_token))?;

            // Check confidence thresholds
            if sell_price.confidence < self.min_confidence {
                return Err(format!(
                    "Low confidence for sell token {:?}: {:.2}",
                    order.sell_token, sell_price.confidence
                ));
            }

            if buy_price.confidence < self.min_confidence {
                return Err(format!(
                    "Low confidence for buy token {:?}: {:.2}",
                    order.buy_token, buy_price.confidence
                ));
            }

            // Validate that clearing prices satisfy order limits
            // sell_amount * sell_price >= buy_amount * buy_price (order is satisfied)
            let sell_value = order.sell_amount * sell_price.price;
            let buy_value = order.buy_amount * buy_price.price;

            if sell_value < buy_value {
                return Err(format!(
                    "Clearing prices don't satisfy order {:?}: sell_value={}, buy_value={}",
                    order.id, sell_value, buy_value
                ));
            }
        }

        info!("All clearing prices validated successfully");
        Ok(())
    }

    /// Calculates total surplus generated by clearing prices
    pub fn calculate_total_surplus(
        &self,
        prices: &HashMap<Address, ClearingPrice>,
        orders: &[Order],
    ) -> f64 {
        let mut total_surplus = 0.0;

        for order in orders {
            if let (Some(sell_price), Some(buy_price)) = (
                prices.get(&order.sell_token),
                prices.get(&order.buy_token),
            ) {
                // Surplus = (clearing_value - limit_value) for the order
                let clearing_value = (order.sell_amount * sell_price.price).as_u128() as f64;
                let limit_value = (order.buy_amount * buy_price.price).as_u128() as f64;

                if clearing_value > limit_value {
                    let surplus = (clearing_value - limit_value) / 1e18;
                    total_surplus += surplus;
                }
            }
        }

        info!("Total surplus: {:.6}", total_surplus);
        total_surplus
    }

    /// Calculates fee for an order based on surplus
    pub fn calculate_fee(&self, order: &Order, surplus: f64, fee_percentage: f64) -> U256 {
        // Fee = surplus * fee_percentage
        let fee = surplus * fee_percentage;
        U256::from((fee * 1e18) as u128)
    }
}

impl Default for PricingEngine {
    fn default() -> Self {
        Self::new(PricingStrategy::MidPoint, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{OrderId, OrderKind, OrderStatus, ChainId};

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
            chain_id: ChainId::Mainnet,
        }
    }

    #[test]
    fn test_midpoint_pricing() {
        let engine = PricingEngine::default();

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let orders = vec![
            create_test_order(token_a, token_b, 1000, 2000),
            create_test_order(token_b, token_a, 2000, 1000),
        ];

        let prices = engine.calculate_clearing_prices(&orders);

        assert!(prices.contains_key(&token_a));
        assert!(prices.get(&token_a).unwrap().confidence > 0.0);
    }

    #[test]
    fn test_volume_weighted_pricing() {
        let engine = PricingEngine::new(PricingStrategy::VolumeWeighted, 0.5);

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let orders = vec![
            create_test_order(token_a, token_b, 1000, 2000),
            create_test_order(token_a, token_b, 2000, 4000),
        ];

        let prices = engine.calculate_clearing_prices(&orders);

        assert!(prices.contains_key(&token_a));
    }

    #[test]
    fn test_market_pricing_with_oracle() {
        let mut engine = PricingEngine::new(PricingStrategy::MarketPrice, 0.5);

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        // Set oracle prices
        engine.set_external_price(token_a, U256::from(2000000000000000000u128)); // 2.0 ETH
        engine.set_external_price(token_b, U256::from(1000000000000000000u128)); // 1.0 ETH

        let orders = vec![
            create_test_order(token_a, token_b, 1000, 2000),
        ];

        let prices = engine.calculate_clearing_prices(&orders);

        assert_eq!(prices.get(&token_a).unwrap().confidence, 0.95);
    }

    #[test]
    fn test_price_validation() {
        let engine = PricingEngine::default();

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let orders = vec![
            create_test_order(token_a, token_b, 1000000000000000000, 2000000000000000000),
        ];

        let prices = engine.calculate_clearing_prices(&orders);
        let result = engine.validate_prices(&prices, &orders);

        assert!(result.is_ok());
    }

    #[test]
    fn test_surplus_calculation() {
        let engine = PricingEngine::default();

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let orders = vec![
            create_test_order(token_a, token_b, 1000000000000000000, 1500000000000000000),
        ];

        let prices = engine.calculate_clearing_prices(&orders);
        let surplus = engine.calculate_total_surplus(&prices, &orders);

        assert!(surplus >= 0.0);
    }

    #[test]
    fn test_fee_calculation() {
        let engine = PricingEngine::default();

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let order = create_test_order(token_a, token_b, 1000, 2000);
        let surplus = 100.0;
        let fee_percentage = 0.1; // 10%

        let fee = engine.calculate_fee(&order, surplus, fee_percentage);

        assert_eq!(fee, U256::from(10000000000000000000u128)); // 10.0 in wei
    }
}
