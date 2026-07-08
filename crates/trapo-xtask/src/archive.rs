use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};

use crate::manifest::NativeDependencyTarget;

pub(crate) fn download_if_needed(
    base_url: &str,
    target: &NativeDependencyTarget,
    archive: &Path,
) -> Result<()> {
    if archive.is_file() && validate_archive_sha(target, archive).is_ok() {
        return Ok(());
    }
    let url = format!("{base_url}/{}", target.asset);
    let response = reqwest::blocking::Client::new()
        .get(&url)
        .header(reqwest::header::USER_AGENT, "trapo-native-deps")
        .send()
        .with_context(|| format!("failed to download {url}"))?
        .error_for_status()
        .with_context(|| format!("native dependency download failed: {url}"))?;
    let mut destination =
        File::create(archive).with_context(|| format!("failed to create {}", archive.display()))?;
    io::copy(&mut response.take(u64::MAX), &mut destination)?;
    if let Err(error) = validate_archive_sha(target, archive) {
        let _ = fs::remove_file(archive);
        return Err(error);
    }
    Ok(())
}

pub(crate) fn validate_archive_sha(target: &NativeDependencyTarget, archive: &Path) -> Result<()> {
    let actual = sha256_file(archive)?;
    if actual != target.sha256 {
        bail!(
            "SHA256 mismatch for {}: expected {}, got {actual}",
            target.asset,
            target.sha256
        );
    }
    Ok(())
}

pub(crate) fn extract_archive(
    target: &NativeDependencyTarget,
    archive: &Path,
    destination: &Path,
) -> Result<()> {
    match target.archive_type.as_str() {
        "zip" => extract_zip(archive, destination),
        "tar.gz" => extract_targz(archive, destination),
        other => bail!("unsupported native dependency archive type: {other}"),
    }
}

fn extract_zip(archive: &Path, destination: &Path) -> Result<()> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        let Some(enclosed) = entry.enclosed_name() else {
            bail!("refusing unsafe zip member: {}", entry.name());
        };
        let target = bounded_destination(destination, &enclosed)?;
        if entry.is_dir() {
            fs::create_dir_all(target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut output = File::create(target)?;
            io::copy(&mut entry, &mut output)?;
        }
    }
    Ok(())
}

fn extract_targz(archive: &Path, destination: &Path) -> Result<()> {
    let file = File::open(archive)?;
    let decoder = GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        let target = bounded_destination(destination, &path)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        entry.unpack(target)?;
    }
    Ok(())
}

fn bounded_destination(root: &Path, relative: &Path) -> Result<PathBuf> {
    let target = root.join(relative);
    let resolved_root = root.canonicalize()?;
    let resolved_parent = target
        .parent()
        .unwrap_or(root)
        .canonicalize()
        .unwrap_or_else(|_| resolved_root.clone());
    if !resolved_parent.starts_with(&resolved_root) {
        bail!(
            "refusing archive member outside destination: {}",
            relative.display()
        );
    }
    Ok(target)
}

pub(crate) fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use anyhow::{Result, anyhow};
    use zip::write::SimpleFileOptions;

    use super::*;
    use crate::native_deps::test_target;

    #[test]
    fn bounded_destination_rejects_parent_escape() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let err = bounded_destination(temp.path(), Path::new("../escape.txt")).err();
        assert!(err.is_some());
        Ok(())
    }

    #[test]
    fn validate_archive_sha_rejects_mismatch() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let archive = temp.path().join("native.zip");
        fs::write(&archive, b"not-the-archive")?;
        let target = test_target("native.zip", "zip", "missing-sha");

        let err = validate_archive_sha(&target, &archive)
            .err()
            .ok_or_else(|| anyhow!("SHA mismatch was accepted"))?;

        assert!(err.to_string().contains("SHA256 mismatch"));
        Ok(())
    }

    #[test]
    fn extract_zip_rejects_parent_escape() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let archive = temp.path().join("unsafe.zip");
        {
            let file = File::create(&archive)?;
            let mut writer = zip::ZipWriter::new(file);
            writer.start_file("../escape.txt", SimpleFileOptions::default())?;
            writer.write_all(b"escape")?;
            writer.finish()?;
        }

        let err = extract_zip(&archive, temp.path())
            .err()
            .ok_or_else(|| anyhow!("unsafe zip member was extracted"))?;

        assert!(err.to_string().contains("unsafe zip member"));
        Ok(())
    }
}
