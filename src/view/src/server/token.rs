//! Token generation and validation

use uuid::Uuid;

/// Manages authentication tokens for the sharing server
pub struct TokenManager {
    token: String,
}

impl TokenManager {
    /// Create a new token manager with a random token
    pub fn new() -> Self {
        Self {
            token: Uuid::new_v4().to_string().replace("-", ""),
        }
    }

    /// Get the current token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Validate a token string
    pub fn validate(&self, candidate: &str) -> bool {
        !candidate.is_empty() && candidate == self.token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let tm = TokenManager::new();
        assert!(!tm.token().is_empty());
        assert_eq!(tm.token().len(), 32); // UUID v4 without hyphens
    }

    #[test]
    fn test_token_validation() {
        let tm = TokenManager::new();
        let token = tm.token().to_string();
        assert!(tm.validate(&token));
        assert!(!tm.validate("wrong_token"));
        assert!(!tm.validate(""));
    }
}
