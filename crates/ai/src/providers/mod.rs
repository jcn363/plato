//! Provider implementations module

pub mod mock;
pub mod ollama;

// Future providers (cloud)
// pub mod openai;
// pub mod claude;

pub use mock::MockProvider;
pub use ollama::OllamaProvider;
