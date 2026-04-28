mod ollama;

pub use ollama::*;

use anyhow::Result;

/// Trait for LLM clients
pub trait LlmClient: Send + Sync {
    /// Generate a summary for the given code/content
    fn generate_summary(&self, content: &str, context: &str) -> Result<String>;

    /// Check if the LLM is available
    fn is_available(&self) -> bool;

    /// Get the model name
    fn model_name(&self) -> &str;
}

/// Create an LLM client based on provider name
pub fn create_client(
    provider: &str,
    endpoint: Option<&str>,
    model: &str,
) -> Result<Box<dyn LlmClient>> {
    match provider.to_lowercase().as_str() {
        "ollama" => {
            let endpoint = endpoint.unwrap_or("http://localhost:11434");
            Ok(Box::new(OllamaClient::new(endpoint, model)?))
        }
        _ => anyhow::bail!("Unknown LLM provider: {}", provider),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client_unknown_provider() {
        let result = create_client("unknown_provider", None, "model");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Unknown LLM provider"));
    }

    #[test]
    fn test_create_client_ollama() {
        // This will create a client but it may not be available
        let result = create_client("ollama", None, "llama3.3");
        // Should succeed in creating the client object
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_case_insensitive() {
        let result1 = create_client("Ollama", None, "llama3.3");
        let result2 = create_client("OLLAMA", None, "llama3.3");

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
