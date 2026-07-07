use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub(in crate::app::ocr_engines) struct RunnerBinary {
    pub(in crate::app::ocr_engines) path: PathBuf,
    pub(in crate::app::ocr_engines) runtime_bin_dir: Option<PathBuf>,
}

pub(in crate::app::ocr_engines) fn find_runner_binary(
    app_root: &Path,
    runtime_id: &str,
    names: &[&str],
) -> Option<RunnerBinary> {
    runner_search_dirs(app_root, runtime_id)
        .into_iter()
        .find_map(|dir| {
            find_binary_in_dir(&dir, names).map(|path| RunnerBinary {
                path,
                runtime_bin_dir: (dir.file_name().and_then(|name| name.to_str()) == Some("bin"))
                    .then_some(dir),
            })
        })
}

pub(in crate::app::ocr_engines) fn runner_binary_is_installed(
    app_root: &Path,
    runtime_id: &str,
    names: &[&str],
) -> bool {
    runner_search_dirs(app_root, runtime_id)
        .iter()
        .any(|dir| find_binary_in_dir(dir, names).is_some())
}

pub(super) fn runner_search_dirs(app_root: &Path, runtime_id: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let runtime_root = app_root.join("thirdparty").join("uocr-runtime");
    if !runtime_id.is_empty() {
        dirs.push(runtime_root.join(runtime_id).join("bin"));
        dirs.push(
            app_root
                .join("thirdparty")
                .join("native-runners")
                .join(runtime_id)
                .join("bin"),
        );
    }
    dirs.push(app_root.join("bin"));
    if let Ok(entries) = std::fs::read_dir(&runtime_root) {
        dirs.extend(
            entries
                .flatten()
                .map(|entry| entry.path().join("bin"))
                .filter(|path| path.is_dir()),
        );
    }
    if let Some(path) = env::var_os("PATH") {
        dirs.extend(env::split_paths(&path));
    }
    let mut unique = Vec::with_capacity(dirs.len());
    let mut seen = std::collections::HashSet::new();
    for dir in dirs {
        if seen.insert(dir.clone()) {
            unique.push(dir);
        }
    }
    unique
}

fn find_binary_in_dir(dir: &Path, names: &[&str]) -> Option<PathBuf> {
    names
        .iter()
        .flat_map(|name| executable_names(name))
        .map(|name| dir.join(name))
        .find(|path| path.is_file())
}

fn executable_names(name: &str) -> Vec<OsString> {
    let mut names = vec![OsString::from(name)];
    let has_exe_extension = Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
    if cfg!(windows) && !has_exe_extension {
        names.push(OsString::from(format!("{name}.exe")));
    }
    names
}
