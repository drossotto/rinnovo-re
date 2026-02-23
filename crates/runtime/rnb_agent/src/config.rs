use std::env;

/// Minimal configuration for the agent.
///
/// This will grow as we add engine management, hardware probing,
/// and workspace policies, but for now it just proves out how we
/// load a registrar URL from the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Base URL for the registrar HTTP API.
    pub registrar_url: String,
}

impl Config {
    /// Construct configuration from process environment with a
    /// sensible default.
    pub fn from_env() -> Self {
        let registrar_url = env::var("RINNOVO_REGISTRAR_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());

        Self { registrar_url }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::env;

    #[test]
    fn default_registrar_url_is_localhost() {
        // Ensure we don't inherit any real test environment override.
        env::remove_var("RINNOVO_REGISTRAR_URL");

        let cfg = Config::from_env();
        assert_eq!(cfg.registrar_url, "http://localhost:8000");
    }

    #[test]
    fn registrar_url_respects_env_override() {
        env::set_var("RINNOVO_REGISTRAR_URL", "https://rinnovo-re.onrender.com");

        let cfg = Config::from_env();
        assert_eq!(cfg.registrar_url, "https://rinnovo-re.onrender.com");
    }
}

