//! Schema of the third-party licence inventory embedded into the web binary.
//!
//! The document is produced by [`cargo-about`] from `apps/web/about.toml` and
//! `apps/web/about.hbs`, embedded by `apps/web/build.rs` and rendered by the
//! `/licenses` route. This module is the contract between those three: the
//! template writes this shape, the build script copies it in unchanged, and the
//! page deserialises it here.
//!
//! It holds no prose. Licence *texts* are reproduced verbatim as their authors
//! wrote them — that is the point of the page — and every label around them
//! comes from the translation files.
//!
//! [`cargo-about`]: https://github.com/EmbarkStudios/cargo-about

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The generated licence inventory: one entry per licence in [`Self::summary`],
/// one per distinct licence *file* in [`Self::texts`], one per dependency in
/// [`Self::crates`].
///
/// Stored normalised — a licence text appears once, however many dependencies
/// ship it — and joined back into one row per dependency by
/// [`Self::dependencies`], which is the shape the page renders. Keeping the
/// document normalised is what holds it to 340 KB in the binary: the same data
/// denormalised, with each shared Apache text repeated under every crate using
/// it, is half again as large.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LicensesFile {
    /// Every licence the dependency set resolves to, with the number of crates
    /// under it. Ordered by cargo-about, most-used first.
    #[serde(default)]
    pub summary: Vec<LicenseSummary>,
    /// Every distinct licence file found, each naming the crates it covers.
    ///
    /// Distinct by text, not by identifier: two crates under MIT ship two files
    /// that differ in their copyright line, and reproducing that line is what MIT
    /// asks for. Hence 100-odd entries over a handful of identifiers.
    ///
    /// `used_by` is the join, not something the page prints — [`Self::dependencies`]
    /// inverts it so each dependency carries its own texts.
    #[serde(default)]
    pub texts: Vec<LicenseText>,
    /// Every crate in the resolved graph, as cargo resolved it for the targets
    /// in `about.toml` — including this repository's own, which
    /// [`Self::third_party`] filters out.
    #[serde(default)]
    pub crates: Vec<CrateLicense>,
}

impl LicensesFile {
    /// The dependencies that came from somewhere else, which is what the page
    /// attributes.
    ///
    /// A crate resolved from a registry or a git remote carries a `source`; one
    /// resolved from a path — this repository's own `web`, `portfolio-data` and
    /// `portfolio-config` — does not. That is the whole test, and it is done here
    /// rather than by naming those three in `about.toml`: a fourth crate in this
    /// workspace then needs no configuration change to stay off a page about
    /// third parties.
    ///
    /// Filtering here rather than in the generator is deliberate too. cargo-about
    /// can drop unpublished crates itself, but it keys that off `publish = false`
    /// on *any* crate rather than on workspace membership, which also drops
    /// third-party dependencies pinned by git tag — see the note in `about.toml`.
    pub fn third_party(&self) -> impl Iterator<Item = &CrateLicense> {
        self.crates.iter().filter(|c| c.source.is_some())
    }

    /// Every third-party dependency with the licence texts that cover it, in the
    /// document's own order.
    ///
    /// The inversion of [`LicenseText::used_by`], done once here rather than by
    /// scanning every text for every dependency while rendering — that is 175
    /// texts against 335 dependencies on each server render, for a join whose
    /// answer never changes within a build.
    ///
    /// A dependency usually has exactly one text. It has several when it is
    /// licensed `A AND B`, or when it vendors code under notices of its own:
    /// `ring` carries eighteen, which are eighteen genuinely different copyright
    /// notices rather than eighteen copies of one.
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

    /// `true` when there is nothing to attribute — a `cargo check` or a `cargo
    /// test` outside the image build, where `cargo about` has not run and
    /// `build.rs` embedded the empty default. The page renders its unavailable
    /// state.
    pub fn is_empty(&self) -> bool {
        self.third_party().next().is_none()
    }
}

/// One dependency and the licence texts it ships under: a row of the page.
#[derive(Clone, Debug, PartialEq)]
pub struct DependencyLicenses<'a> {
    /// The crate being attributed.
    pub dependency: &'a CrateLicense,
    /// Empty only if the generator found no licence file for it at all, which is
    /// worth showing as such rather than hiding the dependency.
    pub texts: Vec<&'a LicenseText>,
}

/// One licence and how many crates resolve to it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LicenseSummary {
    /// SPDX short identifier, e.g. `MIT`.
    pub id: String,
    /// Full name, e.g. `MIT License`.
    pub name: String,
    /// Crates resolving to this licence. Sums to more than the number of
    /// dependencies: a crate licensed `A AND B` is counted by both.
    pub crates: usize,
}

/// One distinct licence text and the crates it covers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LicenseText {
    /// SPDX short identifier this text was identified as.
    pub id: String,
    /// Full name of that licence.
    pub name: String,
    /// The licence file, verbatim — copyright line, wrapping and all. Rendered
    /// preformatted; never reflowed, translated or summarised.
    pub text: String,
    /// The crates this exact text was found in.
    #[serde(default)]
    pub used_by: Vec<CrateRef>,
}

/// A crate named from a licence text's coverage list.
///
/// Name and version together, because a dependency graph can hold two versions of the same crate
/// under two different copyright lines.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrateRef {
    /// Crate name as published.
    pub name: String,
    /// The exact version cargo resolved.
    pub version: String,
}

/// One crate in the resolved dependency graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrateLicense {
    /// Crate name as published.
    pub name: String,
    /// The exact version cargo resolved, which is what the page prints beside the name.
    pub version: String,
    /// The SPDX *expression* the crate declares, e.g. `MIT OR Apache-2.0` —
    /// what it offers, which is not always the single licence it was resolved
    /// under. Both are shown: the offer here, the resolved text in
    /// [`LicensesFile::texts`].
    pub license: String,
    /// Where cargo resolved the crate from: a registry index, a git remote, or
    /// nothing at all for a path dependency. What makes a crate third-party —
    /// see [`LicensesFile::third_party`].
    #[serde(default)]
    pub source: Option<String>,
    /// Upstream repository. Absent for the rare crate that declares none, in
    /// which case the page renders the name unlinked rather than a dead anchor.
    ///
    /// The manifest's `authors` field is deliberately not carried alongside it.
    /// It is empty for a tenth of this graph and stale for more, and the
    /// attribution that carries legal weight is the copyright notice inside the
    /// licence text, which [`LicensesFile::texts`] reproduces in full. Every
    /// field in this document is rendered; a field the page would not show has no
    /// reason to be embedded in the binary.
    #[serde(default)]
    pub repository: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The empty default `build.rs` embeds when the generator has not run has to
    /// deserialise, or every build outside the image would fail to compile the
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
        assert!(parsed.texts[0].used_by.is_empty());
        assert_eq!(parsed.crates[0].repository, None);
    }

    /// The workspace's own crates arrive in the document with no source and the
    /// licence `Unknown`; a page about third parties must not list them, and a
    /// document holding nothing else must report itself empty.
    #[test]
    fn path_dependencies_are_not_third_party() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"summary":[],"texts":[],"crates":[
                {"name":"web","version":"2.6.0","license":"Unknown","source":null},
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
                  {"name":"web","version":"2.6.0","license":"Unknown"}]}"#,
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
    /// as no inventory at all — otherwise the page would render three headings
    /// over an empty list instead of saying there is nothing to show.
    #[test]
    fn an_inventory_of_only_path_dependencies_is_empty() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"summary":[],"texts":[],
                "crates":[{"name":"web","version":"2.6.0","license":"Unknown"}]}"#,
        )
        .expect("valid");

        assert!(parsed.is_empty());
    }

    /// A licence text is reproduced byte for byte; the round-trip is what proves
    /// no serialisation step here rewrites one.
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

        let json = serde_json::to_string(&file).expect("serialises");
        let back: LicensesFile = serde_json::from_str(&json).expect("deserialises");
        assert_eq!(back.texts[0].text, text);
    }
}
