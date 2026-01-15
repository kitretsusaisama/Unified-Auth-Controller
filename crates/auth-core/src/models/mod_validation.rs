// Re-export validation module
pub mod validation;

pub use validation::{normalize_phone, validate_email, detect_identifier_type};
