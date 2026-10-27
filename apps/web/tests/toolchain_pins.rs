//! Versions that are written down in more than one file and have to agree.
//!
//! The Dioxus CLI pin and the Dioxus library version have to be the same release. `dx` writes
//! the asset manifest and the hydration bootstrap that the `dioxus` crates read, so a CLI one
//! release behind the library builds an image that compiles and fails in the browser. The two
//! live in different files — the CLI in the `Dockerfile`, the library in `Cargo.lock` — and the
//! install instructions repeat the CLI version twice more.
//!
//! The Rust toolchain is named three times: `channel` in `rust-toolchain.toml`, the
//! `rust:<version>-slim` base image in the `Dockerfile`, and `rust-version` in `Cargo.toml`.
//! Renovate's `rust toolchain` group moves all three in one pull request.
//!
//! The root `Cargo.toml` states every `[workspace.package]` value a second time in its own
//! `[package]` table, because the README payload action reads that table alone. Each member
//! states the workspace version, because release-please refuses `version.workspace = true`.
//!
//! These tests are what notice when one copy moves without the others.

use std::path::Path;

/// The workspace root, two levels above this crate.
fn workspace_file(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// The version `Cargo.lock` resolves `package` to.
fn locked_version(lock: &str, package: &str) -> String {
    let lock = lock.replace("\r\n", "\n");
    let needle = format!("name = \"{package}\"\nversion = \"");
    let start = lock
        .find(&needle)
        .unwrap_or_else(|| panic!("{package} is not in Cargo.lock"));
    let rest = &lock[start + needle.len()..];
    rest[..rest.find('"').expect("a closing quote")].to_owned()
}

#[test]
fn the_dioxus_cli_pin_matches_the_locked_library() {
    let dockerfile = workspace_file("Dockerfile");
    let cli = dockerfile
        .lines()
        .find_map(|line| line.strip_prefix("ARG DIOXUS_CLI_VERSION="))
        .expect("the Dockerfile pins DIOXUS_CLI_VERSION")
        .trim()
        .to_owned();
    let library = locked_version(&workspace_file("Cargo.lock"), "dioxus");
    assert_eq!(
        cli, library,
        "DIOXUS_CLI_VERSION in the Dockerfile must equal the dioxus version in Cargo.lock"
    );

    let install = format!("cargo install --locked dioxus-cli --version {cli}");
    for doc in [
        "README.md",
        "CONTRIBUTING.md",
        ".github/templates/README.md.hbs",
    ] {
        assert!(
            workspace_file(doc).contains(&install),
            "{doc} must tell contributors to run `{install}`"
        );
    }
}

/// The value of the first `key = "…"` line in a TOML file, read the way `locked_version` reads
/// `Cargo.lock`: anchored to the line start, so a comment or a nested table cannot match.
fn toml_string(file: &str, key: &str) -> String {
    let prefix = format!("{key} = \"");
    let rest = file
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("no top-level `{key}` string"));
    rest[..rest.find('"').expect("a closing quote")].to_owned()
}

/// `major.minor` of a `major.minor.patch` release; anything shorter is returned unchanged.
fn minor_release(version: &str) -> &str {
    match version.match_indices('.').nth(1) {
        Some((end, _)) => &version[..end],
        None => version,
    }
}

#[test]
fn the_rust_toolchain_pin_matches_the_base_image_and_rust_version() {
    let channel = toml_string(&workspace_file("rust-toolchain.toml"), "channel");
    let dockerfile = workspace_file("Dockerfile");
    let image = dockerfile
        .lines()
        .filter(|line| line.starts_with("FROM "))
        .find_map(|line| line.split_once(" rust:"))
        .and_then(|(_, tag)| tag.split_once('-'))
        .map(|(version, _)| version.to_owned())
        .expect("the Dockerfile builds FROM a rust:<version>-slim image");
    let rust_version = toml_string(&workspace_file("Cargo.toml"), "rust-version");

    assert_eq!(
        minor_release(&channel),
        image,
        "channel {channel} in rust-toolchain.toml must be a release of the Dockerfile's rust:{image} base image"
    );
    assert_eq!(
        rust_version, image,
        "rust-version in Cargo.toml must equal the Dockerfile's rust:{image} base image"
    );
}

/// The value of `key = "…"` inside the `[table]` section of a TOML file, or `None` when the table
/// has no such line. Reads line by line, the way [`toml_string`] does: no TOML parser is a
/// dependency of this crate, and these manifests keep one key per line.
fn table_string(file: &str, table: &str, key: &str) -> Option<String> {
    let header = format!("[{table}]");
    let prefix = format!("{key} = \"");
    file.lines()
        .skip_while(|line| line.trim() != header)
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|rest| rest[..rest.find('"').expect("a closing quote")].to_owned())
}

#[test]
fn the_root_package_repeats_the_workspace_package() {
    let manifest = workspace_file("Cargo.toml");
    for key in ["edition", "rust-version", "homepage"] {
        let workspace = table_string(&manifest, "workspace.package", key)
            .unwrap_or_else(|| panic!("[workspace.package] sets no `{key}`"));
        let root = table_string(&manifest, "package", key)
            .unwrap_or_else(|| panic!("the root [package] sets no `{key}`"));
        assert_eq!(
            root, workspace,
            "`{key}` in the root [package] must equal `{key}` in [workspace.package]"
        );
    }
}

#[test]
fn every_member_carries_the_workspace_version() {
    let manifest = workspace_file("Cargo.toml");
    let version =
        table_string(&manifest, "package", "version").expect("the root [package] has a version");
    let members: Vec<&str> = manifest
        .lines()
        .skip_while(|line| line.trim() != "members = [")
        .skip(1)
        .take_while(|line| line.trim() != "]")
        .filter_map(|line| line.trim().strip_prefix('"')?.strip_suffix("\","))
        .collect();
    assert!(
        !members.is_empty(),
        "the root Cargo.toml lists no workspace members"
    );
    for member in members {
        let member_manifest = workspace_file(&format!("{member}/Cargo.toml"));
        assert_eq!(
            table_string(&member_manifest, "package", "version").as_deref(),
            Some(version.as_str()),
            "{member}/Cargo.toml must carry the workspace version {version}"
        );
    }
}
