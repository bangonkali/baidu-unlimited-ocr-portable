use std::{env, path::PathBuf};

use trapo_server::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("openapi/trapo.openapi.json"));
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&ApiDoc::openapi())?;
    std::fs::write(output, format!("{json}\n"))?; // skylos: ignore[SKY-D215] output is an explicit CLI argument for the OpenAPI export tool.
    Ok(())
}
