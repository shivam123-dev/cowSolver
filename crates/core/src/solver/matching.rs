use crate::domain::{Order, OrderId};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Represents a match between orders
#[derive(Debug, Clone)]
pub struct OrderMatch {
    /// Orders involved in the match
    pub orders: Vec<OrderId>,
    
    /// Match type
    pub match_type: MatchType,
    
    /// Quality score (higher is better)
    pub quality_score: f64,
    
    /// Estimated surplus generated
    pub estimated_surplus: f64,
}

/// Type of order match
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchType {
    /// Direct pair match (A sells X for Y, B sells Y for X)
    DirectPair,
    
    /// Ring match (A->B->C->A)
    Ring,
    
    /// Batch match (multiple orders with overlapping tokens)
    Batch,
}

/// Order matching engine
pub struct MatchingEngine {
    /// Maximum ring size to consider
    max_ring_size: usize,
    
    /// Minimum quality score to accept
    min_quality_score: f64,
}

impl MatchingEngine {
    /// Creates a new matching engine
    pub fn new(max_ring_size: usize, min_quality_score: f64) -> Self {
        Self {
            max_ring_size,
            min_quality_score,
        }
    }

    /// Finds all possible matches in a batch of orders
    pub fn find_matches(&self, orders: &[Order]) -> Vec<OrderMatch> {
        let mut matches = Vec::new();

        // Find direct pair matches
        matches.extend(self.find_direct_pairs(orders));

        // Find ring matches
        matches.extend(self.find_rings(orders));

        // Sort by quality score (descending)
        matches.sort_by(|a, b| {
            b.quality_score
                .partial_cmp(&a.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Filter by minimum quality
        matches.retain(|m| m.quality_score >= self.min_quality_score);

        info!("Found {} total matches", matches.len());
        matches
    }

    /// Finds direct pair matches (A<->B)
    fn find_direct_pairs(&self, orders: &[Order]) -> Vec<OrderMatch> {
        let mut matches = Vec::new();

        for (i, order_a) in orders.iter().enumerate() {
            for order_b in orders.iter().skip(i + 1) {
                if self.is_direct_match(order_a, order_b) {
                    let quality = self.calculate_pair_quality(order_a, order_b);
                    let surplus = self.estimate_pair_surplus(order_a, order_b);

                    matches.push(OrderMatch {
                        orders: vec![order_a.id, order_b.id],
                        match_type: MatchType::DirectPair,
                        quality_score: quality,
                        estimated_surplus: surplus,
                    });

                    debug!(
                        "Direct pair match: {:?} <-> {:?}, quality={:.4}",
                        order_a.id, order_b.id, quality
                    );
                }
            }
        }

        info!("Found {} direct pair matches", matches.len());
        matches
    }

    /// Checks if two orders form a direct match
    fn is_direct_match(&self, order_a: &Order, order_b: &Order) -> bool {
        // Orders match if:
        // 1. A sells what B buys AND A buys what B sells
        // 2. Price overlap exists (both can be satisfied)
        
        if order_a.sell_token != order_b.buy_token {
            return false;
        }
        
        if order_a.buy_token != order_b.sell_token {
            return false;
        }

        // Check price overlap
        self.has_price_overlap(order_a, order_b)
    }

    /// Checks if two orders have overlapping price ranges
    fn has_price_overlap(&self, order_a: &Order, order_b: &Order) -> bool {
        // Calculate limit prices
        // order_a limit price: buy_amount / sell_amount (how much buy token per sell token)
        // order_b limit price: sell_amount / buy_amount (inverse)
        
        let price_a = order_a.buy_amount.as_u128() as f64 / order_a.sell_amount.as_u128() as f64;
        let price_b = order_b.sell_amount.as_u128() as f64 / order_b.buy_amount.as_u128() as f64;
        
        // For a valid match: price_a <= price_b
        // This means order_a is willing to accept less than order_b is offering
        price_a <= price_b
    }

    /// Calculates quality score for a pair match
    fn calculate_pair_quality(&self, order_a: &Order, order_b: &Order) -> f64 {
        // Quality factors:
        // 1. Price overlap (larger overlap = better)
        // 2. Volume (larger volume = better)
        // 3. Balance (similar sizes = better)
        
        let price_a = order_a.buy_amount.as_u128() as f64 / order_a.sell_amount.as_u128() as f64;
        let price_b = order_b.sell_amount.as_u128() as f64 / order_b.buy_amount.as_u128() as f64;
        
        // Price overlap score (0-1)
        let price_overlap = if price_b > 0.0 {
            1.0 - (price_a / price_b).min(1.0)
        } else {
            0.0
        };
        
        // Volume score (normalized)
        let volume_a = order_a.sell_amount.as_u128() as f64;
        let volume_b = order_b.sell_amount.as_u128() as f64;
        let total_volume = volume_a + volume_b;
        let volume_score = (total_volume / 1e18).ln().max(0.0) / 10.0; // Log scale, capped
        
        // Balance score (0-1, 1 = perfectly balanced)
        let balance_score = (volume_a.min(volume_b) / volume_a.max(volume_b)).min(1.0);
        
        // Weighted combination
        let quality = price_overlap * 0.4 + volume_score * 0.3 + balance_score * 0.3;
        
        quality.max(0.0).min(1.0)
    }

    /// Estimates surplus for a pair match
    fn estimate_pair_surplus(&self, order_a: &Order, order_b: &Order) -> f64 {
        // Surplus = difference between limit prices
        let price_a = order_a.buy_amount.as_u128() as f64 / order_a.sell_amount.as_u128() as f64;
        let price_b = order_b.sell_amount.as_u128() as f64 / order_b.buy_amount.as_u128() as f64;
        
        if price_b <= price_a {
            return 0.0;
        }
        
        // Calculate surplus based on volume and price difference
        let volume = order_a.sell_amount.as_u128().min(order_b.buy_amount.as_u128()) as f64;
        let price_diff = price_b - price_a;
        
        (volume * price_diff) / 1e18 // Convert from wei
    }

    /// Finds ring matches (cycles of 3+ orders)
    fn find_rings(&self, orders: &[Order]) -> Vec<OrderMatch> {
        let mut matches = Vec::new();

        if orders.len() < 3 {
            return matches;
        }

        // Build token graph
        let graph = self.build_token_graph(orders);

        // Find cycles in the graph
        let cycles = self.find_cycles(&graph, self.max_ring_size);

        for cycle in cycles {
            if let Some(ring_match) = self.validate_ring(orders, &cycle) {
                matches.push(ring_match);
            }
        }

        info!("Found {} ring matches", matches.len());
        matches
    }

    /// Builds a directed graph of token relationships
    fn build_token_graph(&self, orders: &[Order]) -> HashMap<ethers::types::Address, Vec<usize>> {
        let mut graph: HashMap<ethers::types::Address, Vec<usize>> = HashMap::new();

        for (idx, order) in orders.iter().enumerate() {
            graph
                .entry(order.sell_token)
                .or_insert_with(Vec::new)
                .push(idx);
        }

        graph
    }

    /// Finds cycles in the token graph using DFS
    fn find_cycles(
        &self,
        graph: &HashMap<ethers::types::Address, Vec<usize>>,
        max_size: usize,
    ) -> Vec<Vec<usize>> {
        let mut cycles = Vec::new();
        
        // This is a simplified cycle detection
        // A production implementation would use more sophisticated algorithms
        // like Johnson's algorithm for finding all elementary cycles
        
        // For now, we'll just detect simple 3-cycles
        // TODO: Implement full cycle detection algorithm
        
        cycles
    }

    /// Validates and scores a ring match
    fn validate_ring(&self, orders: &[Order], cycle: &[usize]) -> Option<OrderMatch> {
        if cycle.len() < 3 {
            return None;
        }

        // Validate that the ring forms a valid cycle
        for i in 0..cycle.len() {
            let current = &orders[cycle[i]];
            let next = &orders[cycle[(i + 1) % cycle.len()]];

            // Current order's buy token should be next order's sell token
            if current.buy_token != next.sell_token {
                return None;
            }
        }

        // Calculate quality score for the ring
        let quality = self.calculate_ring_quality(orders, cycle);
        let surplus = self.estimate_ring_surplus(orders, cycle);

        Some(OrderMatch {
            orders: cycle.iter().map(|&i| orders[i].id).collect(),
            match_type: MatchType::Ring,
            quality_score: quality,
            estimated_surplus: surplus,
        })
    }

    /// Calculates quality score for a ring match
    fn calculate_ring_quality(&self, orders: &[Order], cycle: &[usize]) -> f64 {
        // Ring quality based on:
        // 1. Number of orders (more = better, up to a point)
        // 2. Price consistency around the ring
        // 3. Volume balance
        
        let size_score = 1.0 / (cycle.len() as f64).sqrt(); // Prefer smaller rings
        
        // Calculate price product around the ring (should be >= 1 for valid ring)
        let mut price_product = 1.0;
        for &idx in cycle {
            let order = &orders[idx];
            let price = order.buy_amount.as_u128() as f64 / order.sell_amount.as_u128() as f64;
            price_product *= price;
        }
        
        let price_score = if price_product >= 1.0 {
            (price_product - 1.0).min(1.0)
        } else {
            0.0
        };
        
        (size_score + price_score) / 2.0
    }

    /// Estimates surplus for a ring match
    fn estimate_ring_surplus(&self, orders: &[Order], cycle: &[usize]) -> f64 {
        // Simplified surplus calculation for rings
        // Real implementation would solve for optimal clearing prices
        
        let mut total_surplus = 0.0;
        
        for &idx in cycle {
            let order = &orders[idx];
            // Estimate surplus as a fraction of order volume
            total_surplus += (order.sell_amount.as_u128() as f64) * 0.001 / 1e18;
        }
        
        total_surplus
    }

    /// Selects non-overlapping matches to maximize total quality
    pub fn select_optimal_matches(&self, matches: Vec<OrderMatch>) -> Vec<OrderMatch> {
        let mut selected = Vec::new();
        let mut used_orders: HashSet<OrderId> = HashSet::new();

        // Greedy selection: pick highest quality matches that don't overlap
        for match_candidate in matches {
            // Check if any order in this match is already used
            let has_overlap = match_candidate
                .orders
                .iter()
                .any(|order_id| used_orders.contains(order_id));

            if !has_overlap {
                // Mark all orders in this match as used
                for order_id in &match_candidate.orders {
                    used_orders.insert(*order_id);
                }
                selected.push(match_candidate);
            }
        }

        info!(
            "Selected {} non-overlapping matches from {} candidates",
            selected.len(),
            matches.len()
        );

        selected
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new(4, 0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{OrderKind, OrderStatus, ChainId};
    use ethers::types::{Address, U256};

    fn create_test_order(
        id: u8,
        sell_token: Address,
        buy_token: Address,
        sell_amount: u128,
        buy_amount: u128,
    ) -> Order {
        let mut order_id = [0u8; 32];
        order_id[0] = id;

        Order {
            id: OrderId(order_id),
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
    fn test_direct_pair_matching() {
        let engine = MatchingEngine::default();

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let orders = vec![
            create_test_order(1, token_a, token_b, 1000, 2000),
            create_test_order(2, token_b, token_a, 2000, 1000),
        ];

        let matches = engine.find_direct_pairs(&orders);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::DirectPair);
        assert_eq!(matches[0].orders.len(), 2);
    }

    #[test]
    fn test_no_match_different_tokens() {
        let engine = MatchingEngine::default();

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);
        let token_c = Address::from_low_u64_be(3);

        let orders = vec![
            create_test_order(1, token_a, token_b, 1000, 2000),
            create_test_order(2, token_b, token_c, 2000, 3000),
        ];

        let matches = engine.find_direct_pairs(&orders);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_price_overlap_detection() {
        let engine = MatchingEngine::default();

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        // Order A: sell 1000 A for 2000 B (price = 2.0)
        // Order B: sell 2000 B for 1000 A (price = 0.5)
        // These should match because A wants 2.0 and B offers 2.0
        let order_a = create_test_order(1, token_a, token_b, 1000, 2000);
        let order_b = create_test_order(2, token_b, token_a, 2000, 1000);

        assert!(engine.has_price_overlap(&order_a, &order_b));
    }

    #[test]
    fn test_quality_scoring() {
        let engine = MatchingEngine::default();

        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);

        let order_a = create_test_order(1, token_a, token_b, 1000000000000000000, 2000000000000000000);
        let order_b = create_test_order(2, token_b, token_a, 2000000000000000000, 1000000000000000000);

        let quality = engine.calculate_pair_quality(&order_a, &order_b);
        assert!(quality > 0.0);
        assert!(quality <= 1.0);
    }

    #[test]
    fn test_optimal_match_selection() {
        let engine = MatchingEngine::default();

        let mut order_id_1 = [0u8; 32];
        order_id_1[0] = 1;
        let mut order_id_2 = [0u8; 32];
        order_id_2[0] = 2;
        let mut order_id_3 = [0u8; 32];
        order_id_3[0] = 3;

        let matches = vec![
            OrderMatch {
                orders: vec![OrderId(order_id_1), OrderId(order_id_2)],
                match_type: MatchType::DirectPair,
                quality_score: 0.8,
                estimated_surplus: 100.0,
            },
            OrderMatch {
                orders: vec![OrderId(order_id_2), OrderId(order_id_3)],
                match_type: MatchType::DirectPair,
                quality_score: 0.6,
                estimated_surplus: 80.0,
            },
        ];

        let selected = engine.select_optimal_matches(matches);
        
        // Should select only the first match since they share order_id_2
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].quality_score, 0.8);
    }
}
