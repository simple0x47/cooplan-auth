use std::time::Instant;

#[derive(Clone)]
pub struct Token {
    token: String,
    generated_at: Instant,
    validity_in_seconds: u64,
}

impl Token {
    pub fn new(token: String, generated_at: Instant, validity_in_seconds: u64) -> Token {
        Token {
            token,
            generated_at,
            validity_in_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.generated_at.elapsed().as_secs() > self.validity_in_seconds
    }

    pub fn value(&self) -> &str {
        &self.token
    }
}
