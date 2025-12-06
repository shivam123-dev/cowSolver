use serde::{Deserialize, Serialize};
use ethers::types::{Address, U256};
use super::chains::ChainId;

/// Represents a token on a specific chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Token {
    /// Token contract address
    pub address: Address,
    
    /// Chain where token exists
    pub chain_id: ChainId,
    
    /// Token symbol (e.g., "USDC", "ETH")
    pub symbol: String,
    
    /// Token name
    pub name: String,
    
    /// Number of decimals
    pub decimals: u8,
}

/// Token amount with decimal awareness
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenAmount {
    /// Raw amount in smallest unit
    pub raw: U256,
    
    /// Number of decimals
    pub decimals: u8,
}

impl TokenAmount {
    /// Creates a new token amount
    pub fn new(raw: U256, decimals: u8) -> Self {
        Self { raw, decimals }
    }
    
    /// Creates token amount from human-readable value
    pub fn from_decimal(value: f64, decimals: u8) -> Self {
        let multiplier = 10_u128.pow(decimals as u32);
        let raw = U256::from((value * multiplier as f64) as u128);
        Self { raw, decimals }
    }
    
    /// Converts to human-readable decimal value
    pub fn to_decimal(&self) -> f64 {
        let divisor = 10_u128.pow(self.decimals as u32) as f64;
        self.raw.as_u128() as f64 / divisor
    }
    
    /// Checks if amount is zero
    pub fn is_zero(&self) -> bool {
        self.raw.is_zero()
    }
    
    /// Adds two token amounts (must have same decimals)
    pub fn checked_add(&self, other: &TokenAmount) -> Option<TokenAmount> {
        if self.decimals != other.decimals {
            return None;
        }
        
        self.raw.checked_add(other.raw).map(|raw| TokenAmount {
            raw,
            decimals: self.decimals,
        })
    }
    
    /// Subtracts two token amounts (must have same decimals)
    pub fn checked_sub(&self, other: &TokenAmount) -> Option<TokenAmount> {
        if self.decimals != other.decimals {
            return None;
        }
        
        self.raw.checked_sub(other.raw).map(|raw| TokenAmount {
            raw,
            decimals: self.decimals,
        })
    }
    
    /// Multiplies amount by a scalar
    pub fn checked_mul(&self, scalar: u128) -> Option<TokenAmount> {
        self.raw.checked_mul(U256::from(scalar)).map(|raw| TokenAmount {
            raw,
            decimals: self.decimals,
        })
    }
    
    /// Divides amount by a scalar
    pub fn checked_div(&self, scalar: u128) -> Option<TokenAmount> {
        if scalar == 0 {
            return None;
        }
        
        self.raw.checked_div(U256::from(scalar)).map(|raw| TokenAmount {
            raw,
            decimals: self.decimals,
        })
    }
}

impl Token {
    /// Creates a new token
    pub fn new(
        address: Address,
        chain_id: ChainId,
        symbol: String,
        name: String,
        decimals: u8,
    ) -> Self {
        Self {
            address,
            chain_id,
            symbol,
            name,
            decimals,
        }
    }
    
    /// Creates a token amount for this token
    pub fn amount(&self, raw: U256) -> TokenAmount {
        TokenAmount::new(raw, self.decimals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_amount_from_decimal() {
        let amount = TokenAmount::from_decimal(1.5, 18);
        assert_eq!(amount.to_decimal(), 1.5);
    }
    
    #[test]
    fn test_token_amount_addition() {
        let a = TokenAmount::new(U256::from(100), 18);
        let b = TokenAmount::new(U256::from(50), 18);
        let result = a.checked_add(&b).unwrap();
        assert_eq!(result.raw, U256::from(150));
    }
    
    #[test]
    fn test_token_amount_subtraction() {
        let a = TokenAmount::new(U256::from(100), 18);
        let b = TokenAmount::new(U256::from(50), 18);
        let result = a.checked_sub(&b).unwrap();
        assert_eq!(result.raw, U256::from(50));
    }
    
    #[test]
    fn test_token_amount_different_decimals() {
        let a = TokenAmount::new(U256::from(100), 18);
        let b = TokenAmount::new(U256::from(50), 6);
        assert!(a.checked_add(&b).is_none());
    }
    
    #[test]
    fn test_token_amount_multiplication() {
        let amount = TokenAmount::new(U256::from(100), 18);
        let result = amount.checked_mul(2).unwrap();
        assert_eq!(result.raw, U256::from(200));
    }
    
    #[test]
    fn test_token_amount_division() {
        let amount = TokenAmount::new(U256::from(100), 18);
        let result = amount.checked_div(2).unwrap();
        assert_eq!(result.raw, U256::from(50));
    }
}
