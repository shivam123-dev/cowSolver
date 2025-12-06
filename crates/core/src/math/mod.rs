use ethers::types::U256;

/// Calculates price impact for a swap
pub fn calculate_price_impact(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
) -> f64 {
    if reserve_in.is_zero() || reserve_out.is_zero() {
        return 0.0;
    }
    
    let amount_in_f = amount_in.as_u128() as f64;
    let reserve_in_f = reserve_in.as_u128() as f64;
    let reserve_out_f = reserve_out.as_u128() as f64;
    
    // Constant product formula: x * y = k
    let k = reserve_in_f * reserve_out_f;
    let new_reserve_in = reserve_in_f + amount_in_f;
    let new_reserve_out = k / new_reserve_in;
    
    let amount_out = reserve_out_f - new_reserve_out;
    let expected_price = reserve_out_f / reserve_in_f;
    let actual_price = amount_out / amount_in_f;
    
    ((expected_price - actual_price) / expected_price).abs()
}

/// Calculates output amount for constant product AMM
pub fn calculate_amm_output(
    amount_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    fee_bps: u32,
) -> Option<U256> {
    if reserve_in.is_zero() || reserve_out.is_zero() {
        return None;
    }
    
    // Apply fee (fee_bps is in basis points, e.g., 30 = 0.3%)
    let fee_multiplier = 10000 - fee_bps;
    let amount_in_with_fee = amount_in
        .checked_mul(U256::from(fee_multiplier))?
        .checked_div(U256::from(10000))?;
    
    // Calculate output: (amount_in_with_fee * reserve_out) / (reserve_in + amount_in_with_fee)
    let numerator = amount_in_with_fee.checked_mul(reserve_out)?;
    let denominator = reserve_in.checked_add(amount_in_with_fee)?;
    
    numerator.checked_div(denominator)
}

/// Calculates required input for desired output (constant product AMM)
pub fn calculate_amm_input(
    amount_out: U256,
    reserve_in: U256,
    reserve_out: U256,
    fee_bps: u32,
) -> Option<U256> {
    if reserve_in.is_zero() || reserve_out.is_zero() || amount_out >= reserve_out {
        return None;
    }
    
    // Calculate input: (reserve_in * amount_out) / ((reserve_out - amount_out) * fee_multiplier)
    let numerator = reserve_in.checked_mul(amount_out)?.checked_mul(U256::from(10000))?;
    let fee_multiplier = 10000 - fee_bps;
    let denominator = reserve_out
        .checked_sub(amount_out)?
        .checked_mul(U256::from(fee_multiplier))?;
    
    numerator.checked_div(denominator)
}

/// Calculates optimal split for routing through multiple paths
pub fn calculate_optimal_split(
    amount: U256,
    path_reserves: Vec<(U256, U256)>,
) -> Vec<U256> {
    // Simplified: equal split for now
    // TODO: Implement proper optimization based on reserves
    let num_paths = path_reserves.len();
    if num_paths == 0 {
        return vec![];
    }
    
    let split_amount = amount / U256::from(num_paths);
    vec![split_amount; num_paths]
}

/// Calculates geometric mean price
pub fn geometric_mean_price(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    
    let product: f64 = prices.iter().product();
    product.powf(1.0 / prices.len() as f64)
}

/// Calculates weighted average price
pub fn weighted_average_price(prices: &[(f64, f64)]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    
    let total_weight: f64 = prices.iter().map(|(_, w)| w).sum();
    if total_weight == 0.0 {
        return 0.0;
    }
    
    let weighted_sum: f64 = prices.iter().map(|(p, w)| p * w).sum();
    weighted_sum / total_weight
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_amm_output_calculation() {
        let amount_in = U256::from(1000);
        let reserve_in = U256::from(100000);
        let reserve_out = U256::from(100000);
        let fee_bps = 30; // 0.3%
        
        let output = calculate_amm_output(amount_in, reserve_in, reserve_out, fee_bps);
        assert!(output.is_some());
        assert!(output.unwrap() < amount_in); // Should get less due to fees
    }
    
    #[test]
    fn test_price_impact() {
        let amount_in = U256::from(1000);
        let reserve_in = U256::from(100000);
        let reserve_out = U256::from(100000);
        
        let impact = calculate_price_impact(amount_in, reserve_in, reserve_out);
        assert!(impact > 0.0);
        assert!(impact < 1.0);
    }
    
    #[test]
    fn test_geometric_mean() {
        let prices = vec![1.0, 2.0, 4.0];
        let mean = geometric_mean_price(&prices);
        assert!((mean - 2.0).abs() < 0.01);
    }
    
    #[test]
    fn test_weighted_average() {
        let prices = vec![(100.0, 1.0), (200.0, 2.0)];
        let avg = weighted_average_price(&prices);
        assert!((avg - 166.67).abs() < 0.1);
    }
}
