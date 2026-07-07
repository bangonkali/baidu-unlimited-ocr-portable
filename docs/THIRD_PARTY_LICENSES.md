# Third-Party License Summary

Last reviewed: 2026-07-08 (UTC)

This note summarizes third-party license obligations for the current project layout
in `baidu-unlimited-ocr-portable`.

## Primary project license

- Project/workbench crates: AGPL-3.0-or-later
  - `src/trapo-server/Cargo.toml`
  - `crates/trapo-downloader/Cargo.toml`
  - `crates/trapo-native-runners/Cargo.toml`
  - Root workspace `Cargo.toml` (`license = "AGPL-3.0-or-later"`)

## Copyleft and restricted licenses

- `pdfium` (Rust crate from `Cargo.toml`): **GPL-3.0**
- `thirdparty/PDFium-rs` (`Cargo.toml`, included as project source): **GPL-3.0**
- All AGPL packages above are already covered by this repository license policy.

## Most common transitive Rust and frontend license families

- Permissive + permissive-like: **MIT**, **Apache-2.0**, **ISC**, **BSD-2-Clause**,
  **BSD-3-Clause**, **MIT OR Apache-2.0**, **MIT/Apache-2.0**, **Unlicense**,
  **BlueOak-1.0.0**, **CC0-1.0**
- Additional notices seen in transitive stacks:
  - **MPL-2.0** (e.g. `axe-core`, `lightningcss*`)
  - **CC-BY-4.0** (e.g. `caniuse-lite`)
  - **Python-2.0** (e.g. `argparse`)
  - **Unlicense** (minor transitive packages)

Because source and lock files are large, treat this file as a summary. For exact
full text and versions at release time:

- Rust: use `cargo metadata --format-version 1 --all-features` and `Cargo.lock`.
- Frontend: use `src/trapo-client/bun.lock` and package metadata in `node_modules`.
- Vendored thirdparty: read `LICENSE*` files under `thirdparty/*`.

## Minimum compliance artifacts to ship

1. `LICENSE`
2. `NOTICE`
3. `docs/AGPL_COMPLIANCE.md`
4. `docs/THIRD_PARTY_LICENSES.md`

## Verification quick checklist

- Ensure `LICENSE` text is present and unmodified.
- Ensure all release archives include third-party notice files.
- Confirm the source offer for AGPL network use is in your user-facing packaging notes
  (or website) and points to the exact source for that release.

