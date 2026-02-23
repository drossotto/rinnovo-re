use rnb_agent::Config;

/// Very small integration-style check that we can construct
/// configuration from the ambient environment. This is a placeholder
/// for richer end-to-end tests once the agent starts the engine and
/// registrar interactions.
#[test]
fn config_from_env_smoke_test() {
    let cfg = Config::from_env();
    assert!(!cfg.registrar_url.is_empty());
}

