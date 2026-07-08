//! Exports the generated Trapo `OpenAPI` document to a JSON file.

use std::{env, path::PathBuf};

use trapo_server::openapi_document;

fn main() -> anyhow::Result<()> {
    let output = env::args().nth(1).map_or_else(
        || PathBuf::from("openapi/trapo.openapi.json"),
        PathBuf::from,
    );
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&openapi_document())?;
    std::fs::write(output, format!("{json}\n"))?; // skylos: ignore[SKY-D215] output is an explicit CLI argument for the OpenAPI export tool.
    Ok(())
}
