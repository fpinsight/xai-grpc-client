//! Client implementation for the xAI Grok API.
//!
//! This module contains the main client interface ([`GrokClient`]) and configuration
//! ([`GrokConfig`]) for interacting with the Grok API.

// Module organization for maintainability
// Each submodule focuses on a specific concern

mod config;
mod conversions;
mod operations;

// Re-export public API
pub use config::{GrokClient, GrokConfig};
