use anyhow::Result;
use rnb_agent::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Minimal stub: load configuration and print it so we can
    // confirm the daemon wiring
    let cfg = Config::from_env();
    println!(
        "rnb daemon starting with registrar at {}",
        cfg.registrar_url
    );
    Ok(())
}

