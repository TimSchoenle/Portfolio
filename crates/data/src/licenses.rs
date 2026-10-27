//! The schema of the third-party license inventory embedded into the web binary.
//!
//! [`cargo-about`] produces the document from `apps/web/about.toml` and `apps/web/about.hbs`,
//! `apps/web/build.rs` embeds it, and the `/licenses` route renders it. This module is the
//! contract between those three: the template writes this shape, the build script copies it in
//! unchanged, and the page deserializes it here.
//!
//! It holds no prose. License texts are reproduced verbatim as their authors wrote them, and
//! every label around them comes from the translation files.
//!
//! # Why the document is normalized
//!
//! A license text appears once in [`LicensesFile::texts`], however many dependencies ship it, and
//! [`LicensesFile::dependencies`] joins it back to one row per dependency. That holds the
//! document to about 340 KB in the binary; repeating each shared Apache text under every crate
//! using it makes it half again as large. The join runs once per call over 175 texts and 335
//! dependencies, where scanning every text for every dependency during each render would repeat
//! a join whose answer never changes within a build.
//!
//! # Why the workspace's own crates are filtered here
//!
//! A crate resolved from a path carries no `source`, and that is the whole test in
//! [`LicensesFile::third_party`]. Naming the workspace crates in `about.toml` would need an edit
//! for every new one. cargo-about can drop unpublished crates itself, but it keys that off
//! `publish = false` on any crate, which also drops third-party dependencies pinned by git tag;
//! see the note in `about.toml`.
//!
//! # Why there is no `authors` field
//!
//! It is empty for a tenth of the graph and stale for more, and the attribution that carries
//! legal weight is the copyright notice inside the license text, which is reproduced in full.
//! Every field in this document is rendered, so a field the page would not show is not embedded.
//!
//! [`cargo-about`]: https://github.com/EmbarkStudios/cargo-about

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The generated license inventory.
///
/// Each license text appears once, however many dependencies ship it. Read the rows the page
/// renders through [`Self::dependencies`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LicensesFile {
    /// Every license the dependency set resolves to, most-used first.
    #[serde(default)]
    pub summary: Vec<LicenseSummary>,
    /// Every distinct license file found, each naming the crates it covers.
    ///
    /// Distinct by text, not by identifier: two crates under MIT ship two files that differ in
    /// their copyright line, and reproducing that line is what MIT asks for.
    #[serde(default)]
    pub texts: Vec<LicenseText>,
    /// Every crate cargo resolved for the targets in `about.toml`, including the workspace's own.
    ///
    /// [`Self::third_party`] leaves the workspace's crates out.
    #[serde(default)]
    pub crates: Vec<CrateLicense>,
}

impl LicensesFile {
    /// Returns the crates resolved from a registry or a git remote, which the page attributes.
    ///
    /// A crate resolved from a path, which is every crate of this workspace, is left out.
    pub fn third_party(&self) -> impl Iterator<Item = &CrateLicense> {
        self.crates.iter().filter(|c| c.source.is_some())
    }

    /// Returns every third-party dependency with the license texts that cover it, in the
    /// document's order.
    ///
    /// A dependency usually has exactly one text. It has several when it is licensed `A AND B`,
    /// or when it vendors code under notices of its own: `ring` carries eighteen different
    /// copyright notices.
    #[must_use]
    pub fn dependencies(&self) -> Vec<DependencyLicenses<'_>> {
        let mut covered: BTreeMap<(&str, &str), Vec<&LicenseText>> = BTreeMap::new();
        for text in &self.texts {
            for used in &text.used_by {
                covered
                    .entry((used.name.as_str(), used.version.as_str()))
                    .or_default()
                    .push(text);
            }
        }

        self.third_party()
            .map(|dependency| DependencyLicenses {
                texts: covered
                    .remove(&(dependency.name.as_str(), dependency.version.as_str()))
                    .unwrap_or_default(),
                dependency,
            })
            .collect()
    }

    /// Reports whether there is no third-party crate to attribute.
    ///
    /// That is the case outside the image build, where `cargo about` has not run and `build.rs`
    /// embedded the empty default.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.third_party().next().is_none()
    }
}

/// One dependency and the license texts it ships under: a row of the page.
#[derive(Clone, Debug, PartialEq)]
pub struct DependencyLicenses<'a> {
    /// The crate being attributed.
    pub dependency: &'a CrateLicense,
    /// The license texts covering the crate, empty when the generator found none.
    ///
    /// The page shows such a dependency with no text instead of hiding it.
    pub texts: Vec<&'a LicenseText>,
}

/// One license and how many crates resolve to it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LicenseSummary {
    /// The SPDX short identifier, such as `MIT`.
    pub id: String,
    /// The full name, such as `MIT License`.
    pub name: String,
    /// The number of crates resolving to this license.
    ///
    /// The counts sum to more than the number of dependencies: a crate licensed `A AND B` is
    /// counted by both.
    pub crates: usize,
}

/// One distinct license text and the crates it covers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LicenseText {
    /// The SPDX short identifier this text was identified as.
    pub id: String,
    /// The full name of that license.
    pub name: String,
    /// The license file verbatim, copyright line and wrapping included.
    ///
    /// The page renders it preformatted, never reflowed, translated or summarized.
    pub text: String,
    /// The crates this exact text was found in.
    #[serde(default)]
    pub used_by: Vec<CrateRef>,
}

/// A crate named in a license text's coverage list.
///
/// Both fields identify it, because a dependency graph can hold two versions of the same crate
/// under two different copyright lines.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrateRef {
    /// The crate name as published.
    pub name: String,
    /// The exact version cargo resolved.
    pub version: String,
}

/// One crate in the resolved dependency graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrateLicense {
    /// The crate name as published.
    pub name: String,
    /// The exact version cargo resolved, which the page prints beside the name.
    pub version: String,
    /// The SPDX expression the crate declares, such as `MIT OR Apache-2.0`.
    ///
    /// This is what the crate offers, which is not always the license it was resolved under. The
    /// page shows both: this expression, and the resolved text from [`LicensesFile::texts`].
    pub license: String,
    /// The registry index or git remote cargo resolved the crate from, or `None` for a path
    /// dependency.
    ///
    /// A crate with a source is third-party; see [`LicensesFile::third_party`].
    #[serde(default)]
    pub source: Option<String>,
    /// The upstream repository, or `None` when the crate declares none.
    ///
    /// The page renders the name of a crate without one unlinked.
    #[serde(default)]
    pub repository: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The empty default `build.rs` embeds when the generator has not run has to
    /// deserialize, or every build outside the image would fail to compile the
    /// page rather than render it empty.
    #[test]
    fn the_empty_default_parses_and_reports_itself_empty() {
        let parsed: LicensesFile =
            serde_json::from_str(r#"{"summary":[],"texts":[],"crates":[]}"#).expect("valid");
        assert!(parsed.is_empty());
        assert_eq!(parsed, LicensesFile::default());
    }

    /// Absent optional fields must not fail the whole document: a crate that
    /// declares no repository is not rare, and losing the whole inventory over
    /// one would take the page down with it.
    #[test]
    fn optional_crate_fields_default_rather_than_fail() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"summary":[{"id":"MIT","name":"MIT License","crates":1}],
                "texts":[{"id":"MIT","name":"MIT License","text":"Copyright (c) 2020 Someone"}],
                "crates":[{"name":"anyhow","version":"1.0.0","license":"MIT OR Apache-2.0",
                           "source":"registry+https://github.com/rust-lang/crates.io-index"}]}"#,
        )
        .expect("valid");

        assert!(!parsed.is_empty());
        assert_eq!(parsed.summary[0].crates, 1);
        assert_eq!(parsed.texts[0].used_by, Vec::<CrateRef>::new());
        assert_eq!(parsed.crates[0].repository, None);
    }

    /// The workspace's own crates arrive in the document with no source and the
    /// license `Unknown`; a page about third parties must not list them, and a
    /// document holding nothing else must report itself empty.
    #[test]
    fn path_dependencies_are_not_third_party() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"summary":[],"texts":[],"crates":[
                {"name":"portfolio-web","version":"2.6.0","license":"Unknown","source":null},
                {"name":"anyhow","version":"1.0.0","license":"MIT",
                 "source":"registry+https://github.com/rust-lang/crates.io-index"},
                {"name":"terrace-config","version":"0.9.0","license":"MIT",
                 "source":"git+https://github.com/TimSchoenle/terrace-config?tag=v0.9.0"}]}"#,
        )
        .expect("valid");

        let third_party: Vec<&str> = parsed.third_party().map(|c| c.name.as_str()).collect();
        assert_eq!(third_party, ["anyhow", "terrace-config"]);
        assert!(!parsed.is_empty());
    }

    /// The join the page reads: every dependency carries the texts naming it,
    /// a shared text reaches every dependency that ships it, and a dependency
    /// with several notices keeps all of them.
    #[test]
    fn dependencies_carry_the_texts_that_name_them() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"summary":[],
                "texts":[
                  {"id":"MIT","name":"MIT License","text":"shared",
                   "used_by":[{"name":"a","version":"1.0.0"},{"name":"b","version":"2.0.0"}]},
                  {"id":"ISC","name":"ISC License","text":"vendored",
                   "used_by":[{"name":"b","version":"2.0.0"}]}],
                "crates":[
                  {"name":"a","version":"1.0.0","license":"MIT","source":"registry+x"},
                  {"name":"b","version":"2.0.0","license":"MIT AND ISC","source":"registry+x"},
                  {"name":"c","version":"3.0.0","license":"MIT","source":"registry+x"},
                  {"name":"portfolio-web","version":"2.6.0","license":"Unknown"}]}"#,
        )
        .expect("valid");

        let rows = parsed.dependencies();
        let named: Vec<&str> = rows.iter().map(|r| r.dependency.name.as_str()).collect();
        assert_eq!(
            named,
            ["a", "b", "c"],
            "path dependencies stay off the page"
        );

        assert_eq!(rows[0].texts.len(), 1, "a shared text reaches each user");
        assert_eq!(rows[0].texts[0].text, "shared");

        let both: Vec<&str> = rows[1].texts.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            both,
            ["MIT", "ISC"],
            "every notice is kept, in document order"
        );

        assert!(
            rows[2].texts.is_empty(),
            "a dependency no text names is still listed"
        );
    }

    /// An inventory holding nothing but this repository's own crates is as empty
    /// as no inventory at all. Otherwise the page would render three headings
    /// over an empty list instead of saying there is nothing to show.
    #[test]
    fn an_inventory_of_only_path_dependencies_is_empty() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"summary":[],"texts":[],
                "crates":[{"name":"portfolio-web","version":"2.6.0","license":"Unknown"}]}"#,
        )
        .expect("valid");

        assert!(parsed.is_empty());
    }

    /// A license text is reproduced byte for byte; the round-trip is what proves
    /// no serialization step here rewrites one.
    #[test]
    fn licence_text_survives_a_round_trip_verbatim() {
        let text = "MIT License\n\nCopyright (c) 2020 A. Person <a@example.com>\n\n\"Software\"\n";
        let file = LicensesFile {
            summary: Vec::new(),
            texts: vec![LicenseText {
                id: "MIT".into(),
                name: "MIT License".into(),
                text: text.into(),
                used_by: vec![CrateRef {
                    name: "example".into(),
                    version: "1.0.0".into(),
                }],
            }],
            crates: Vec::new(),
        };

        let json = serde_json::to_string(&file).expect("serializes");
        let back: LicensesFile = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back.texts[0].text, text);
    }
}
