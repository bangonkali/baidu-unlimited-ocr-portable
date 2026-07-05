//! Guards the database query coverage manifest.

use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn db_manifest_covers_storage_methods_routes_and_query_count() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("could not resolve repository root"))?;
    let repo_manifest = repo_root.join("docs").join("DB.md");
    let manifest = fs::read_to_string(&repo_manifest)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", repo_manifest.display()))?;

    for method in storage_methods(&manifest_dir.join("src").join("storage"))? {
        assert!(
            manifest.contains(&format!("`{method}`")),
            "DB.md is missing storage method `{method}`"
        );
    }
    for route in route_paths(&manifest_dir.join("src").join("routes.rs"))? {
        assert!(
            manifest.contains(&route),
            "DB.md is missing API route {route}"
        );
    }

    let storage_query_count =
        runtime_storage_query_count(&manifest_dir.join("src").join("storage"))?;
    assert!(
        manifest.contains(&format!(
            "Runtime storage query call sites: `{storage_query_count}`"
        )),
        "DB.md must list runtime storage query call site count {storage_query_count}"
    );
    let migration_count = migration_bundle_count(
        &manifest_dir
            .join("src")
            .join("storage")
            .join("migrations.rs"),
    )?;
    assert!(
        manifest.contains(&format!("Migration SQL bundles: `{migration_count}`")),
        "DB.md must list migration SQL bundle count {migration_count}"
    );
    Ok(())
}

fn storage_methods(storage_dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut methods = Vec::new();
    for entry in fs::read_dir(storage_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if matches!(
            file_name,
            "coverage_tests.rs" | "migration_tests.rs" | "test_fixtures.rs"
        ) {
            continue;
        }
        for line in fs::read_to_string(&path)?.lines() {
            if let Some(name) = public_method_name(line)
                && name != "path"
            {
                methods.push(name);
            }
        }
    }
    methods.sort();
    methods.dedup();
    Ok(methods)
}

fn public_method_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    for prefix in [
        "pub async fn ",
        "pub(crate) async fn ",
        "pub(crate) fn ",
        "pub(super) fn ",
    ] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest
                .split('(')
                .next()
                .filter(|value| !value.is_empty())
                .map(ToString::to_string);
        }
    }
    None
}

fn route_paths(routes_file: &Path) -> anyhow::Result<Vec<String>> {
    let mut routes = Vec::new();
    let mut pending_route = false;
    let source = fs::read_to_string(routes_file)?; // skylos: ignore[SKY-D215] routes_file is resolved from CARGO_MANIFEST_DIR and points to the repository route source.
    for line in source.lines() {
        if line.contains(".route(") {
            pending_route = true;
        }
        if pending_route && let Some(path) = first_quoted_value(line) {
            routes.push(path);
            pending_route = false;
        }
    }
    routes.sort();
    routes.dedup();
    Ok(routes)
}

fn first_quoted_value(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn runtime_storage_query_count(storage_dir: &Path) -> anyhow::Result<usize> {
    let skip = [
        "coverage_tests.rs",
        "migration_tests.rs",
        "test_fixtures.rs",
        "helpers.rs",
        "records.rs",
        "diagnostics_rows.rs",
        "diagnostics_types.rs",
        "migrations.rs",
    ];
    let mut count = 0;
    for entry in fs::read_dir(storage_dir)? {
        let path = entry?.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if path.extension().and_then(|value| value.to_str()) != Some("rs")
            || skip.contains(&file_name)
        {
            continue;
        }
        for line in fs::read_to_string(&path)?.lines() {
            if [".execute(", ".prepare(", ".query_row(", ".execute_batch("]
                .iter()
                .any(|marker| line.contains(marker))
            {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn migration_bundle_count(migrations_file: &Path) -> anyhow::Result<usize> {
    let source = fs::read_to_string(migrations_file)?; // skylos: ignore[SKY-D215] migrations_file is resolved from CARGO_MANIFEST_DIR and points to the repository migration source.
    Ok(source
        .lines()
        .filter(|line| line.trim_start().starts_with("Migration {"))
        .count())
}
