use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub output_dir: PathBuf,
    pub use_llm: bool,
    pub llm_endpoint: String,
    pub llm_model: String,
    pub generation: GenerationConfig,
    pub staleness: StalenessConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub architecture_max_tokens: usize,
    pub module_max_tokens: usize,
    pub include_hotspots: bool,
    pub include_dependencies: bool,
    pub hotspot_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessConfig {
    pub auto_update: bool,
    pub max_age_hours: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from(".ai/context"),
            use_llm: false,
            llm_endpoint: "http://localhost:11434".to_string(),
            llm_model: "llama3.3".to_string(),
            generation: GenerationConfig::default(),
            staleness: StalenessConfig::default(),
        }
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            architecture_max_tokens: 500,
            module_max_tokens: 300,
            include_hotspots: true,
            include_dependencies: true,
            hotspot_limit: 10,
        }
    }
}

impl Default for StalenessConfig {
    fn default() -> Self {
        Self {
            auto_update: true,
            max_age_hours: 24,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.output_dir, PathBuf::from(".ai/context"));
        assert!(!config.use_llm);
        assert_eq!(config.llm_endpoint, "http://localhost:11434");
    }

    #[test]
    fn test_default_generation_config() {
        let config = GenerationConfig::default();
        assert_eq!(config.architecture_max_tokens, 500);
        assert_eq!(config.module_max_tokens, 300);
        assert!(config.include_hotspots);
    }

    #[test]
    fn test_default_staleness_config() {
        let config = StalenessConfig::default();
        assert!(config.auto_update);
        assert_eq!(config.max_age_hours, 24);
    }
}
