use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZettelId(String);

impl ZettelId {
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        if Self::is_valid(&id) {
            Ok(ZettelId(id))
        } else {
            Err(format!("Invalid Zettel ID: {}", id))
        }
    }
    
    pub fn generate_luhmann_style(parent: Option<&str>) -> Self {
        match parent {
            None => ZettelId("1".to_string()),
            Some(parent_id) => ZettelId(format!("{}a", parent_id)),
        }
    }
    
    pub fn is_valid(id: &str) -> bool {
        !id.is_empty() && id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn parse(s: &str) -> Result<Self, String> {
        Self::new(s)
    }
    
    pub fn is_luhmann_style(&self) -> bool {
        let chars: Vec<char> = self.0.chars().collect();
        if chars.is_empty() {
            return false;
        }
        
        if !chars[0].is_ascii_digit() {
            return false;
        }
        
        let mut expect_digit = true;
        for c in &chars[1..] {
            if expect_digit && !c.is_ascii_digit() && !c.is_ascii_lowercase() {
                return false;
            }
            if !expect_digit && !c.is_ascii_lowercase() && !c.is_ascii_digit() {
                return false;
            }
            expect_digit = !expect_digit;
        }
        
        true
    }
    
    pub fn parent(&self) -> Option<Self> {
        if self.0.len() <= 1 {
            return None;
        }
        
        let parent_str = &self.0[..self.0.len() - 1];
        if parent_str.is_empty() {
            return None;
        }
        
        Some(ZettelId(parent_str.to_string()))
    }
    
    pub fn depth(&self) -> usize {
        self.0.len() - 1
    }
}

impl fmt::Display for ZettelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ZettelId {
    fn default() -> Self {
        Self::generate_luhmann_style(None)
    }
}