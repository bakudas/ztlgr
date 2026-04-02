use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Id<T> {
    value: String,
    #[serde(skip)]
    _marker: std::marker::PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new() -> Self {
        Self::from_uuid(uuid::Uuid::new_v4())
    }

    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self {
            value: uuid.to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        if s.is_empty() {
            return Err("ID cannot be empty".to_string());
        }
        Ok(Self {
            value: s.to_string(),
            _marker: std::marker::PhantomData,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> From<&str> for Id<T> {
    fn from(s: &str) -> Self {
        Self::parse(s).unwrap_or_else(|_| Self::new())
    }
}

impl<T> From<String> for Id<T> {
    fn from(s: String) -> Self {
        Self::parse(&s).unwrap_or_else(|_| Self::new())
    }
}
