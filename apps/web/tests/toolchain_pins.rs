//! The Dioxus CLI pin and the Dioxus library version have to be the same release.
//!
//! `dx` writes the asset manifest and the hydration bootstrap that the `dioxus` crates read, so
//! a CLI one release behind the library builds an image that compiles and fails in the
//! browser. The two live in different files — the CLI in the `Dockerfile`, the library in
//! `Cargo.lock` — and the install instructions repeat the CLI version twice more, so this test
//! is what notices when one of them moves without the others.

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
