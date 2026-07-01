use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=DUCKDB_DOWNLOAD_LIB");
    println!("cargo:rerun-if-env-changed=TRAPO_GIT_TAG");
    println!("cargo:rerun-if-env-changed=TRAPO_GIT_SHA");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    emit_version_env();
    emit_platform_link_args();
    let Some(runtime_lib) = duckdb_runtime_library() else {
        return Ok(());
    };
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let Some(profile_dir) = profile_dir(&out_dir) else {
        println!(
            "cargo:warning=Could not determine target profile directory from {}",
            out_dir.display()
        );
        return Ok(());
    };
    let Some(source) = find_runtime_library(&profile_dir, runtime_lib)? else {
        println!("cargo:warning=Could not find {runtime_lib} for trapo-server runtime copy");
        return Ok(());
    };
    let destination = profile_dir.join(runtime_lib);
    copy_if_changed(&source, &destination)?;
    println!(
        "cargo:warning=Copied {} to {}",
        source.display(),
        destination.display()
    );
    Ok(())
}

fn emit_platform_link_args() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg-bin=trapo-server=-Wl,-rpath,@executable_path");
    }
}

fn emit_version_env() {
    let git_tag = env::var("TRAPO_GIT_TAG")
        .ok()
        .or_else(|| env::var("GITHUB_REF_NAME").ok())
        .or_else(|| git_output(["describe", "--tags", "--dirty", "--always"]));
    let git_sha = env::var("TRAPO_GIT_SHA")
        .ok()
        .or_else(|| env::var("GITHUB_SHA").ok())
        .or_else(|| git_output(["rev-parse", "--short=12", "HEAD"]));

    if let Some(value) = git_tag {
        println!("cargo:rustc-env=TRAPO_GIT_TAG={value}");
    }
    if let Some(value) = git_sha {
        println!("cargo:rustc-env=TRAPO_GIT_SHA={value}");
    }
}

fn git_output<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn duckdb_runtime_library() -> Option<&'static str> {
    match env::var("CARGO_CFG_TARGET_OS").ok()?.as_str() {
        "windows" => Some("duckdb.dll"),
        "macos" => Some("libduckdb.dylib"),
        "linux" => Some("libduckdb.so"),
        _ => None,
    }
}

fn profile_dir(out_dir: &Path) -> Option<PathBuf> {
    let build_dir = out_dir.ancestors().find(|ancestor| {
        ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "build")
    })?;
    build_dir.parent().map(Path::to_path_buf)
}

fn find_runtime_library(
    profile_dir: &Path,
    runtime_lib: &str,
) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let deps_candidate = profile_dir.join("deps").join(runtime_lib);
    if deps_candidate.is_file() {
        return Ok(Some(deps_candidate));
    }
    let target_dir = profile_dir.parent().unwrap_or(profile_dir);
    find_under(&target_dir.join("duckdb-download"), runtime_lib)
}

fn find_under(root: &Path, runtime_lib: &str) -> Result<Option<PathBuf>, Box<dyn Error>> {
    if !root.is_dir() {
        return Ok(None);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|name| name.to_str()) == Some(runtime_lib) {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}

fn copy_if_changed(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    if destination.is_file()
        && fs::metadata(source)?.len() == fs::metadata(destination)?.len()
        && fs::read(source)? == fs::read(destination)?
    {
        return Ok(());
    }
    fs::copy(source, destination)?;
    Ok(())
}
