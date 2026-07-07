# AGPL Compliance Notes

This repository is licensed under the GNU Affero General Public License, version 3,
or (at your option) any later version (AGPL-3.0-or-later).

## Files included for compliance

When distributing binaries, source archives, or portable runtime bundles:

- Include `LICENSE` at the package root.
- Include `NOTICE`.
- Include `docs/THIRD_PARTY_LICENSES.md`.

For every release you distribute, these files should be shipped unchanged.

## Network use obligations (AGPL section 13)

If users interact with the program over a network (for example by running a hosted
or shared instance), you must provide a clear offer for access to the complete
corresponding source code used to run that service.

A practical way to satisfy this is to provide:

- A public source URL for the exact release tag/build, and
- A short statement in your product/website UI and distribution notes with the
  source availability link.

For this repository, the source link is:

https://github.com/bangonkali/baidu-unlimited-ocr-portable

## Third-party obligations to keep in sync

Any change to third-party dependencies or vendored thirdparty trees should be
reflected in `docs/THIRD_PARTY_LICENSES.md` before release. In particular, review:

- Rust dependencies in `Cargo.toml`/`Cargo.lock`.
- Frontend dependencies in `src/trapo-client/package.json` and lock file.
- Vendored license files inside `thirdparty/`.

## Recommended distribution wording

You can include this text in release notes or an install README:

> Trapo Workbench is AGPL-3.0-or-later software. The complete source code is
> available at the project repository link above. This release also includes
> third-party components under their respective open-source licenses.

