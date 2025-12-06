use crate::domain::{Order, Token};
use ethers::types::{Address, U256};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use tracing::{debug, info};

/// Represents a liquidity pool
#[derive(Debug, Clone)]
pub struct LiquidityPool {
    /// Pool address
    pub address: Address,
    
    /// Pool type
    pub pool_type: PoolType,
    
    /// Token A
    pub token_a: Address,
    
    /// Token B
    pub token_b: Address,
    
    /// Reserve of token A
    pub reserve_a: U256,
    
    /// Reserve of token B
    pub reserve_b: U256,
    
    /// Pool fee (in basis points, e.g., 30 = 0.3%)
    pub fee_bps: u16,
    
    /// Gas cost to interact with this pool
    pub gas_cost: u64,
}

/// Type of AMM pool
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolType {
    /// Uniswap V2 style (constant product)
    UniswapV2,
    
    /// Uniswap V3 (concentrated liquidity)
    UniswapV3,
    
    /// Balancer weighted pool
    Balancer,
    
    /// Curve stable swap
    Curve,
    
    /// Generic constant product
    ConstantProduct,
}

/// Represents a route through AMM pools
#[derive(Debug, Clone)]
pub struct Route {
    /// Pools in the route
    pub pools: Vec<LiquidityPool>,
    
    /// Tokens in the path (including start and end)
    pub path: Vec<Address>,
    
    /// Expected output amount
    pub output_amount: U256,
    
    /// Total gas cost
    pub gas_cost: u64,
    
    /// Price impact (as percentage)
    pub price_impact: f64,
    
    /// Route quality score
    pub score: f64,
}

/// AMM routing engine
pub struct RoutingEngine {
    /// Available liquidity pools
    pools: Vec<LiquidityPool>,
    
    /// Pool lookup by token pair
    pool_index: HashMap<(Address, Address), Vec<usize>>,
    
    /// Maximum number of hops
    max_hops: usize,
    
    /// Maximum price impact allowed (as percentage)
    max_price_impact: f64,
}

impl RoutingEngine {
    /// Creates a new routing engine
    pub fn new(max_hops: usize, max_price_impact: f64) -> Self {
        Self {
            pools: Vec::new(),
            pool_index: HashMap::new(),
            max_hops,
            max_price_impact,
        }
    }

    /// Adds a liquidity pool to the routing engine
    pub fn add_pool(&mut self, pool: LiquidityPool) {
        let idx = self.pools.len();
        
        // Index by both token orderings
        self.pool_index
            .entry((pool.token_a, pool.token_b))
            .or_insert_with(Vec::new)
            .push(idx);
        
        self.pool_index
            .entry((pool.token_b, pool.token_a))
            .or_insert_with(Vec::new)
            .push(idx);
        
        self.pools.push(pool);
    }

    /// Finds the best route for a swap
    pub fn find_best_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Option<Route> {
        info!(
            "Finding route: {:?} -> {:?}, amount: {}",
            token_in, token_out, amount_in
        );

        // Find all possible routes
        let routes = self.find_all_routes(token_in, token_out, amount_in);

        if routes.is_empty() {
            debug!("No routes found");
            return None;
        }

        // Select best route by score
        let best_route = routes
            .into_iter()
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(Ordering::Equal)
            })?;

        info!(
            "Best route: {} hops, output: {}, score: {:.4}",
            best_route.pools.len(),
            best_route.output_amount,
            best_route.score
        );

        Some(best_route)
    }

    /// Finds all possible routes up to max_hops
    fn find_all_routes(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Vec<Route> {
        let mut routes = Vec::new();

        // Try direct routes (1 hop)
        if let Some(direct_route) = self.find_direct_route(token_in, token_out, amount_in) {
            routes.push(direct_route);
        }

        // Try multi-hop routes if enabled
        if self.max_hops > 1 {
            routes.extend(self.find_multi_hop_routes(token_in, token_out, amount_in));
        }

        // Filter by price impact
        routes.retain(|r| r.price_impact <= self.max_price_impact);

        routes
    }

    /// Finds direct route (single pool)
    fn find_direct_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Option<Route> {
        let pool_indices = self.pool_index.get(&(token_in, token_out))?;

        let mut best_route: Option<Route> = None;

        for &pool_idx in pool_indices {
            let pool = &self.pools[pool_idx];
            
            // Calculate output amount
            let output_amount = self.calculate_output(pool, token_in, amount_in);
            
            if output_amount.is_zero() {
                continue;
            }

            // Calculate price impact
            let price_impact = self.calculate_price_impact(pool, token_in, amount_in);

            // Calculate route score
            let score = self.calculate_route_score(output_amount, pool.gas_cost, price_impact);

            let route = Route {
                pools: vec![pool.clone()],
                path: vec![token_in, token_out],
                output_amount,
                gas_cost: pool.gas_cost,
                price_impact,
                score,
            };

            // Keep best route
            if best_route.is_none() || route.score > best_route.as_ref().unwrap().score {
                best_route = Some(route);
            }
        }

        best_route
    }

    /// Finds multi-hop routes using graph search
    fn find_multi_hop_routes(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Vec<Route> {
        // Use Dijkstra's algorithm to find best paths
        // This is a simplified implementation
        
        let mut routes = Vec::new();
        
        // Build token graph
        let graph = self.build_token_graph();
        
        // Find paths using BFS with limited depth
        let paths = self.find_paths_bfs(&graph, token_in, token_out, self.max_hops);
        
        // Evaluate each path
        for path in paths {
            if let Some(route) = self.evaluate_path(&path, amount_in) {
                routes.push(route);
            }
        }
        
        routes
    }

    /// Builds a graph of token connections
    fn build_token_graph(&self) -> HashMap<Address, Vec<Address>> {
        let mut graph: HashMap<Address, Vec<Address>> = HashMap::new();

        for pool in &self.pools {
            graph
                .entry(pool.token_a)
                .or_insert_with(Vec::new)
                .push(pool.token_b);
            
            graph
                .entry(pool.token_b)
                .or_insert_with(Vec::new)
                .push(pool.token_a);
        }

        graph
    }

    /// Finds paths using breadth-first search
    fn find_paths_bfs(
        &self,
        graph: &HashMap<Address, Vec<Address>>,
        start: Address,
        end: Address,
        max_depth: usize,
    ) -> Vec<Vec<Address>> {
        let mut paths = Vec::new();
        let mut queue = vec![(start, vec![start])];

        while let Some((current, path)) = queue.pop() {
            if path.len() > max_depth {
                continue;
            }

            if current == end && path.len() > 1 {
                paths.push(path.clone());
                continue;
            }

            if let Some(neighbors) = graph.get(&current) {
                for &neighbor in neighbors {
                    // Avoid cycles
                    if !path.contains(&neighbor) {
                        let mut new_path = path.clone();
                        new_path.push(neighbor);
                        queue.push((neighbor, new_path));
                    }
                }
            }
        }

        paths
    }

    /// Evaluates a token path and creates a route
    fn evaluate_path(&self, path: &[Address], amount_in: U256) -> Option<Route> {
        if path.len() < 2 {
            return None;
        }

        let mut pools = Vec::new();
        let mut current_amount = amount_in;
        let mut total_gas = 0u64;
        let mut total_price_impact = 0.0;

        // For each hop in the path
        for i in 0..path.len() - 1 {
            let token_in = path[i];
            let token_out = path[i + 1];

            // Find best pool for this hop
            let pool_indices = self.pool_index.get(&(token_in, token_out))?;
            
            let mut best_pool: Option<&LiquidityPool> = None;
            let mut best_output = U256::zero();

            for &pool_idx in pool_indices {
                let pool = &self.pools[pool_idx];
                let output = self.calculate_output(pool, token_in, current_amount);
                
                if output > best_output {
                    best_output = output;
                    best_pool = Some(pool);
                }
            }

            let pool = best_pool?;
            
            if best_output.is_zero() {
                return None;
            }

            pools.push(pool.clone());
            total_gas += pool.gas_cost;
            total_price_impact += self.calculate_price_impact(pool, token_in, current_amount);
            current_amount = best_output;
        }

        let score = self.calculate_route_score(current_amount, total_gas, total_price_impact);

        Some(Route {
            pools,
            path: path.to_vec(),
            output_amount: current_amount,
            gas_cost: total_gas,
            price_impact: total_price_impact,
            score,
        })
    }

    /// Calculates output amount for a swap through a pool
    fn calculate_output(&self, pool: &LiquidityPool, token_in: Address, amount_in: U256) -> U256 {
        // Determine which direction we're swapping
        let (reserve_in, reserve_out) = if token_in == pool.token_a {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };

        match pool.pool_type {
            PoolType::UniswapV2 | PoolType::ConstantProduct => {
                self.calculate_constant_product_output(amount_in, reserve_in, reserve_out, pool.fee_bps)
            }
            PoolType::UniswapV3 => {
                // Simplified - real implementation would use tick math
                self.calculate_constant_product_output(amount_in, reserve_in, reserve_out, pool.fee_bps)
            }
            PoolType::Balancer => {
                // Simplified - real implementation would use weighted math
                self.calculate_constant_product_output(amount_in, reserve_in, reserve_out, pool.fee_bps)
            }
            PoolType::Curve => {
                // Simplified - real implementation would use StableSwap invariant
                self.calculate_stable_swap_output(amount_in, reserve_in, reserve_out, pool.fee_bps)
            }
        }
    }

    /// Calculates output for constant product formula (x * y = k)
    fn calculate_constant_product_output(
        &self,
        amount_in: U256,
        reserve_in: U256,
        reserve_out: U256,
        fee_bps: u16,
    ) -> U256 {
        if amount_in.is_zero() || reserve_in.is_zero() || reserve_out.is_zero() {
            return U256::zero();
        }

        // amount_in_with_fee = amount_in * (10000 - fee_bps)
        let amount_in_with_fee = amount_in * U256::from(10000 - fee_bps);
        
        // numerator = amount_in_with_fee * reserve_out
        let numerator = amount_in_with_fee * reserve_out;
        
        // denominator = reserve_in * 10000 + amount_in_with_fee
        let denominator = reserve_in * U256::from(10000) + amount_in_with_fee;
        
        if denominator.is_zero() {
            return U256::zero();
        }

        numerator / denominator
    }

    /// Calculates output for stable swap (simplified)
    fn calculate_stable_swap_output(
        &self,
        amount_in: U256,
        reserve_in: U256,
        reserve_out: U256,
        fee_bps: u16,
    ) -> U256 {
        // Simplified stable swap - real implementation would use the full invariant
        // For stable pairs, price impact is much lower
        
        let fee_multiplier = U256::from(10000 - fee_bps);
        let amount_out = amount_in * fee_multiplier / U256::from(10000);
        
        // Cap at reserve
        amount_out.min(reserve_out * U256::from(99) / U256::from(100))
    }

    /// Calculates price impact for a swap
    fn calculate_price_impact(&self, pool: &LiquidityPool, token_in: Address, amount_in: U256) -> f64 {
        let (reserve_in, reserve_out) = if token_in == pool.token_a {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };

        if reserve_in.is_zero() {
            return 100.0; // Max impact
        }

        // Price impact = (amount_in / reserve_in) * 100
        let impact = (amount_in.as_u128() as f64 / reserve_in.as_u128() as f64) * 100.0;
        
        impact.min(100.0)
    }

    /// Calculates route quality score
    fn calculate_route_score(&self, output_amount: U256, gas_cost: u64, price_impact: f64) -> f64 {
        // Score factors:
        // 1. Output amount (higher is better)
        // 2. Gas cost (lower is better)
        // 3. Price impact (lower is better)
        
        let output_score = (output_amount.as_u128() as f64) / 1e18;
        let gas_penalty = (gas_cost as f64) / 1e6; // Normalize gas cost
        let impact_penalty = price_impact / 100.0;
        
        // Weighted score
        output_score - gas_penalty - impact_penalty
    }
}

impl Default for RoutingEngine {
    fn default() -> Self {
        Self::new(3, 5.0) // Max 3 hops, 5% max price impact
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pool(
        token_a: Address,
        token_b: Address,
        reserve_a: u128,
        reserve_b: u128,
    ) -> LiquidityPool {
        LiquidityPool {
            address: Address::zero(),
            pool_type: PoolType::UniswapV2,
            token_a,
            token_b,
            reserve_a: U256::from(reserve_a),
            reserve_b: U256::from(reserve_b),
            fee_bps: 30, // 0.3%
            gas_cost: 100000,
        }
    }

    #[test]
    fn test_constant_product_calculation() {
        let engine = RoutingEngine::default();
        
        let amount_in = U256::from(1000);
        let reserve_in = U256::from(100000);
        let reserve_out = U256::from(200000);
        let fee_bps = 30;
        
        let output = engine.calculate_constant_product_output(
            amount_in,
            reserve_in,
            reserve_out,
            fee_bps,
        );
        
        assert!(output > U256::zero());
        assert!(output < U256::from(2000)); // Should be less than 2x input
    }

    #[test]
    fn test_direct_route() {
        let mut engine = RoutingEngine::default();
        
        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);
        
        let pool = create_test_pool(token_a, token_b, 1000000, 2000000);
        engine.add_pool(pool);
        
        let route = engine.find_best_route(token_a, token_b, U256::from(1000));
        
        assert!(route.is_some());
        let route = route.unwrap();
        assert_eq!(route.pools.len(), 1);
        assert_eq!(route.path.len(), 2);
    }

    #[test]
    fn test_multi_hop_route() {
        let mut engine = RoutingEngine::new(3, 10.0);
        
        let token_a = Address::from_low_u64_be(1);
        let token_b = Address::from_low_u64_be(2);
        let token_c = Address::from_low_u64_be(3);
        
        // Create path A -> B -> C
        engine.add_pool(create_test_pool(token_a, token_b, 1000000, 2000000));
        engine.add_pool(create_test_pool(token_b, token_c, 2000000, 3000000));
        
        let route = engine.find_best_route(token_a, token_c, U256::from(1000));
        
        assert!(route.is_some());
        let route = route.unwrap();
        assert_eq!(route.pools.len(), 2);
        assert_eq!(route.path.len(), 3);
    }

    #[test]
    fn test_price_impact_calculation() {
        let engine = RoutingEngine::default();
        
        let token_a = Address::from_low_u64_be(1);
        let pool = create_test_pool(token_a, Address::from_low_u64_be(2), 1000000, 2000000);
        
        let small_impact = engine.calculate_price_impact(&pool, token_a, U256::from(1000));
        let large_impact = engine.calculate_price_impact(&pool, token_a, U256::from(100000));
        
        assert!(small_impact < large_impact);
        assert!(small_impact < 1.0); // Less than 1% for small trade
        assert!(large_impact > 5.0); // More than 5% for large trade
    }
}
