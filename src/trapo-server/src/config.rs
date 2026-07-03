use std::{
    env,
    path::{Path, PathBuf},
};

const DEFAULT_PORT: u16 = 8765;

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
            .unwrap_or(DEFAULT_PORT);
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
        let log_dir = env::var_os("TRAPO_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| app_root.join("logs"));
        let model_dir = app_root.join("models");
        let database_path = data_dir.join("trapo.duckdb");
        let client_dist = env::var_os("TRAPO_CLIENT_DIST")
            .map(PathBuf::from)
            .unwrap_or_else(|| resolve_client_dist(&app_root));
        let pdfium_library_dir = env::var_os("TRAPO_PDFIUM_DIR")
            .map(PathBuf::from)
            .or_else(|| resolve_pdfium_dir(&app_root));

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
    if let Some(exe_dir) = executable_app_root() {
        return exe_dir;
    }
    source_app_root()
}

fn executable_app_root() -> Option<PathBuf> {
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))?;
    (!is_cargo_target_dir(&exe_dir)).then_some(exe_dir)
}

fn source_app_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn is_cargo_target_dir(path: &Path) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.file_name().is_some_and(|name| name == "target"))
}

fn resolve_client_dist(app_root: &Path) -> PathBuf {
    let packaged = app_root.join("web");
    if packaged.join("index.html").is_file() {
        return packaged;
    }
    app_root.join("src").join("trapo-client").join("dist")
}

fn resolve_pdfium_dir(app_root: &Path) -> Option<PathBuf> {
    let mut candidates = vec![
        app_root.join("thirdparty").join("pdfium").join("bin"),
        app_root.join("thirdparty").join("pdfium").join("lib"),
        app_root.join("thirdparty").join("pdfium"),
        app_root.to_path_buf(),
    ];
    candidates.extend(local_dist_pdfium_dirs(app_root));
    candidates
        .into_iter()
        .find(|path| path.join(pdfium_library_name()).is_file())
}

fn local_dist_pdfium_dirs(app_root: &Path) -> Vec<PathBuf> {
    let dist = app_root.join("dist");
    let Ok(entries) = std::fs::read_dir(dist) else {
        return Vec::new();
    };
    let mut roots = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("trapo-workbench-"))
        })
        .collect::<Vec<_>>();
    roots.sort_by(|left, right| right.file_name().cmp(&left.file_name()));
    roots
        .into_iter()
        .flat_map(|root| {
            [
                root.join("thirdparty").join("pdfium").join("bin"),
                root.join("thirdparty").join("pdfium").join("lib"),
                root.join("thirdparty").join("pdfium"),
            ]
        })
        .collect()
}

fn pdfium_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_packaged_web_before_source_dist() -> std::io::Result<()> {
        let temp = tempfile::tempdir()?;
        let web = temp.path().join("web");
        std::fs::create_dir_all(&web)?;
        std::fs::write(web.join("index.html"), "")?;
        assert_eq!(resolve_client_dist(temp.path()), web);
        Ok(())
    }

    #[test]
    fn resolves_packaged_pdfium_before_dist_fallback() -> std::io::Result<()> {
        let temp = tempfile::tempdir()?;
        let packaged = temp.path().join("thirdparty").join("pdfium").join("bin");
        let fallback = temp
            .path()
            .join("dist")
            .join("trapo-workbench-windows-x64-v0.1.9")
            .join("thirdparty")
            .join("pdfium")
            .join("bin");
        std::fs::create_dir_all(&packaged)?;
        std::fs::create_dir_all(&fallback)?;
        std::fs::write(packaged.join(pdfium_library_name()), "")?;
        std::fs::write(fallback.join(pdfium_library_name()), "")?;
        assert_eq!(
            resolve_pdfium_dir(temp.path()).as_deref(),
            Some(packaged.as_path())
        );
        Ok(())
    }

    #[test]
    fn resolves_local_dist_pdfium_for_source_runs() -> std::io::Result<()> {
        let temp = tempfile::tempdir()?;
        let fallback = temp
            .path()
            .join("dist")
            .join("trapo-workbench-windows-x64-v0.1.9")
            .join("thirdparty")
            .join("pdfium")
            .join("bin");
        std::fs::create_dir_all(&fallback)?;
        std::fs::write(fallback.join(pdfium_library_name()), "")?;
        assert_eq!(
            resolve_pdfium_dir(temp.path()).as_deref(),
            Some(fallback.as_path())
        );
        Ok(())
    }

    #[test]
    fn detects_cargo_target_directories() {
        let target = Path::new("repo").join("target").join("debug");
        assert!(is_cargo_target_dir(&target));
        assert!(!is_cargo_target_dir(Path::new("package")));
    }
}
