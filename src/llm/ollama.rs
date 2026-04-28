use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::LlmClient;

/// Client for Ollama local LLM
pub struct OllamaClient {
    endpoint: String,
    model: String,
    client: reqwest::blocking::Client,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(endpoint: &str, model: &str) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
            client,
        })
    }

    /// Get the API endpoint for generation
    fn generate_url(&self) -> String {
        format!("{}/api/generate", self.endpoint)
    }

    /// Get the API endpoint for checking available models
    fn tags_url(&self) -> String {
        format!("{}/api/tags", self.endpoint)
    }
}

impl LlmClient for OllamaClient {
    fn generate_summary(&self, content: &str, context: &str) -> Result<String> {
        let prompt = format!(
            r#"You are a code documentation assistant. Generate a concise summary of the following code.

Context: {}

Code:
```
{}
```

Provide a brief, technical summary (2-3 sentences) that describes:
1. What this code does
2. Key functions/classes
3. Important patterns or design decisions

Summary:"#,
            context, content
        );

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
        };

        let response = self
            .client
            .post(self.generate_url())
            .json(&request)
            .send()
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Ollama request failed with status: {}",
                response.status()
            );
        }

        let result: OllamaResponse = response
            .json()
            .context("Failed to parse Ollama response")?;

        Ok(result.response.trim().to_string())
    }

    fn is_available(&self) -> bool {
        match self.client.get(self.tags_url()).send() {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://localhost:11434", "llama3.3");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.endpoint, "http://localhost:11434");
        assert_eq!(client.model, "llama3.3");
    }

    #[test]
    fn test_ollama_client_strips_trailing_slash() {
        let client = OllamaClient::new("http://localhost:11434/", "llama3.3").unwrap();
        assert_eq!(client.endpoint, "http://localhost:11434");
    }

    #[test]
    fn test_generate_url() {
        let client = OllamaClient::new("http://localhost:11434", "llama3.3").unwrap();
        assert_eq!(
            client.generate_url(),
            "http://localhost:11434/api/generate"
        );
    }

    #[test]
    fn test_tags_url() {
        let client = OllamaClient::new("http://localhost:11434", "llama3.3").unwrap();
        assert_eq!(client.tags_url(), "http://localhost:11434/api/tags");
    }

    #[test]
    fn test_model_name() {
        let client = OllamaClient::new("http://localhost:11434", "llama3.3").unwrap();
        assert_eq!(client.model_name(), "llama3.3");
    }

    #[test]
    #[ignore = "requires Ollama to be running"]
    fn test_is_available_when_running() {
        let client = OllamaClient::new("http://localhost:11434", "llama3.3").unwrap();
        // This test only passes if Ollama is actually running
        assert!(client.is_available());
    }

    #[test]
    fn test_is_available_wrong_endpoint() {
        let client = OllamaClient::new("http://localhost:99999", "llama3.3").unwrap();
        assert!(!client.is_available());
    }

    #[test]
    fn test_ollama_request_serialization() {
        let request = OllamaRequest {
            model: "llama3.3".to_string(),
            prompt: "test prompt".to_string(),
            stream: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("llama3.3"));
        assert!(json.contains("test prompt"));
        assert!(json.contains("false"));
    }
}
