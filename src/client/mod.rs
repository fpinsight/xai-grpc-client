// Module organization for maintainability
// Each submodule focuses on a specific concern

mod config;
mod conversions;
mod operations;

// Re-export public API
pub use config::{GrokClient, GrokConfig};
