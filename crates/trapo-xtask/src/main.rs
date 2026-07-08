//! Build-time helpers for preparing Trapo native dependencies.

use std::env;

use anyhow::{Result, bail};

mod archive;
mod manifest;
mod native_deps;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command, subcommand, rest @ ..] if command == "native-deps" && subcommand == "prepare" => {
            native_deps::prepare_native_deps(rest)
        }
        [command, subcommand, rest @ ..]
            if command == "native-deps" && subcommand == "check-onnx" =>
        {
            let repo_root = native_deps::repo_root_from_args(rest)?;
            let manifest = manifest::load_manifest(&repo_root)?;
            manifest::check_onnx_submodule(&repo_root, &manifest)
        }
        _ => bail!(
            "usage: trapo-xtask native-deps prepare --platform <id> [--repo-root <path>]\n       trapo-xtask native-deps check-onnx [--repo-root <path>]"
        ),
    }
}
