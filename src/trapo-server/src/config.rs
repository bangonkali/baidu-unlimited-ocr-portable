use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub app_root: PathBuf,
    pub client_dist: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub model_dir: PathBuf,
    pub database_path: PathBuf,
    pub pdfium_library_dir: Option<PathBuf>,
    pub host: String,
    pub port: u16,
    pub open_browser: bool,
}

impl ServerConfig {
    pub fn from_env_and_args<I>(args: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let app_root = env::var_os("TRAPO_APP_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(default_app_root);
        let host = env::var("TRAPO_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let mut port = env::var("TRAPO_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8890);
        let mut open_browser = env::var("TRAPO_NO_BROWSER").is_err();

        let mut iter = args.into_iter().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--port" => {
                    if let Some(value) = iter.next().and_then(|raw| raw.parse().ok()) {
                        port = value;
                    }
                }
                "--no-browser" => open_browser = false,
                _ if arg.starts_with("--port=") => {
                    if let Ok(value) = arg[7..].parse() {
                        port = value;
                    }
                }
                _ => {}
            }
        }

        let data_dir = app_root.join("data");
        let cache_dir = app_root.join("cache");
        let log_dir = app_root.join(".logs");
        let model_dir = app_root.join("models");
        let database_path = data_dir.join("trapo.duckdb");
        let client_dist = app_root.join("src").join("trapo-client").join("dist");
        let pdfium_library_dir = env::var_os("TRAPO_PDFIUM_DIR").map(PathBuf::from);

        Self {
            app_root,
            client_dist,
            data_dir,
            cache_dir,
            log_dir,
            model_dir,
            database_path,
            pdfium_library_dir,
            host,
            port,
            open_browser,
        }
    }

    pub fn ensure_directories(&self) -> std::io::Result<()> {
        for path in [
            &self.data_dir,
            &self.cache_dir,
            &self.log_dir,
            &self.model_dir,
        ] {
            std::fs::create_dir_all(path)?;
        }
        migrate_legacy_database(&self.app_root, &self.database_path)?;
        Ok(())
    }
}

fn default_app_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn migrate_legacy_database(app_root: &Path, database_path: &Path) -> std::io::Result<()> {
    if database_path.exists() {
        return Ok(());
    }
    let legacy = app_root.join("data").join("uocr.duckdb");
    if legacy.exists() {
        std::fs::copy(legacy, database_path)?;
    }
    Ok(())
}
