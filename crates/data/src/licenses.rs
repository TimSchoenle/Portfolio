//! Schema of the third-party licence inventory embedded into the web binary.
//!
//! The document is produced by [`cargo-about`] from `apps/web/about.toml` and
//! `apps/web/about.hbs`, embedded by `apps/web/build.rs` and rendered by the
//! `/licenses` route. This module is the contract between those three: the
//! template writes this shape, the build script copies it in unchanged, and the
//! page deserialises it here.
//!
//! It also holds the two views the page renders — [`LicensesFile::notices`], the
//! attribution proper, and [`LicensesFile::third_party`], the inventory beside
//! it. Both are computed here rather than in the components so the shape the page
//! renders is the shape the tests below pin.
//!
//! It holds no prose. Licence *texts* are reproduced verbatim as their authors
//! wrote them — that is the point of the page — and every label around them
//! comes from the translation files.
//!
//! [`cargo-about`]: https://github.com/EmbarkStudios/cargo-about

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// The generated licence inventory: one entry per distinct licence *file* in
/// [`Self::texts`], one per dependency in [`Self::crates`].
///
/// Stored normalised — a licence text appears once, however many dependencies
/// ship it — and regrouped by licence for rendering by [`Self::notices`].
/// Keeping the document normalised is what holds it to a third of a megabyte in
/// the binary: the same data denormalised, with each shared Apache text repeated
/// under every crate using it, is half again as large.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LicensesFile {
    /// Every distinct licence file found, each naming the crates it covers.
    ///
    /// Distinct by *file*, not by licence and not quite by text: two crates under
    /// MIT ship two files that differ in their copyright line, and reproducing
    /// that line is what MIT asks for. Hence 100-odd entries over a handful of
    /// identifiers — and hence also several entries that are the same notice
    /// wrapped differently, which [`Self::notices`] merges.
    #[serde(default)]
    pub texts: Vec<LicenseText>,
    /// Every crate in the resolved dependency graph, as cargo resolved it for the
    /// targets in `about.toml` — including this repository's own, which
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

    /// The attribution, grouped the way it is read: one entry per licence, and
    /// under it every *distinct* notice with the crates that ship it.
    ///
    /// **The merge is why this exists.** cargo-about reports one entry per licence
    /// file it harvested, so the page's first shape — a row per dependency, each
    /// reproducing its own texts — emitted 499 KB of licence text for 215 KB of
    /// distinct notices, the same MIT paragraph 57 times over. Grouping by licence
    /// and merging identical texts prints each notice once and lists the crates it
    /// covers beneath it, which is also what the licences themselves ask for.
    ///
    /// **The merge is on the text, not on the identifier.** An MIT file *is*
    /// mostly its copyright line, and the 130 distinct MIT notices in this graph
    /// name 130 different copyright holders; collapsing those to one would drop
    /// exactly the part the licence requires be reproduced. Whitespace is folded
    /// for the comparison only — two files differing by a line wrap are one
    /// notice — and the first copy is reproduced verbatim, byte for byte.
    ///
    /// Ordering is total and derived from the data, so the page is stable across
    /// builds that did not change the graph: licences by the number of crates they
    /// cover, then by identifier; notices by the same, then by the crates they
    /// name.
    pub fn notices(&self) -> Vec<LicenseNotices<'_>> {
        // A licence text naming a path dependency is one this repository wrote,
        // and a page about third parties must not reproduce it as though someone
        // else had — the same rule `third_party` applies to the inventory.
        let attributable: BTreeSet<(&str, &str)> = self
            .third_party()
            .map(|c| (c.name.as_str(), c.version.as_str()))
            .collect();

        // (id, name) -> whitespace-folded text -> the notice. `BTreeMap` so the
        // grouping order is a property of the data rather than of a hasher; the
        // vectors below are sorted explicitly regardless.
        let mut by_licence: BTreeMap<(&str, &str), BTreeMap<String, Notice<'_>>> = BTreeMap::new();
        for text in &self.texts {
            let covered: Vec<&CrateRef> = text
                .used_by
                .iter()
                .filter(|c| attributable.contains(&(c.name.as_str(), c.version.as_str())))
                .collect();
            if covered.is_empty() {
                continue;
            }

            by_licence
                .entry((text.id.as_str(), text.name.as_str()))
                .or_default()
                .entry(whitespace_folded(&text.text))
                .or_insert_with(|| Notice {
                    text: text.text.as_str(),
                    crates: Vec::new(),
                })
                .crates
                .extend(covered);
        }

        let mut licences: Vec<LicenseNotices<'_>> = by_licence
            .into_iter()
            .map(|((id, name), merged)| {
                let mut notices: Vec<Notice<'_>> = merged
                    .into_values()
                    .map(|mut notice| {
                        // `&CrateRef` orders by name then version through the
                        // derive, so a crate named by two files of one notice —
                        // the same text under two paths in one crate — collapses.
                        notice.crates.sort_unstable();
                        notice.crates.dedup();
                        notice
                    })
                    .collect();
                notices.sort_by(|a, b| {
                    b.crates
                        .len()
                        .cmp(&a.crates.len())
                        .then_with(|| a.crate_names().cmp(&b.crate_names()))
                });

                LicenseNotices {
                    id,
                    name,
                    // Distinct crates, not the sum over notices: a crate whose
                    // notices differ per vendored file — `ring` carries eighteen —
                    // is one crate under this licence, not eighteen. Counting the
                    // sum is the bug the chips used to render, where "299
                    // dependencies" was really 299 licence files.
                    crates: notices
                        .iter()
                        .flat_map(|n| n.crates.iter().map(|c| (&c.name, &c.version)))
                        .collect::<BTreeSet<_>>()
                        .len(),
                    notices,
                }
            })
            .collect();

        licences.sort_by(|a, b| b.crates.cmp(&a.crates).then_with(|| a.id.cmp(b.id)));
        licences
    }

    /// `true` when there is nothing to attribute — a `cargo check` or a `cargo
    /// test` outside the image build, where `cargo about` has not run and
    /// `build.rs` embedded the empty default. The page renders its unavailable
    /// state.
    pub fn is_empty(&self) -> bool {
        self.third_party().next().is_none()
    }
}

/// One licence and every distinct notice reproduced under it: a section of the
/// page.
#[derive(Clone, Debug, PartialEq)]
pub struct LicenseNotices<'a> {
    /// SPDX short identifier, e.g. `MIT`.
    pub id: &'a str,
    /// Full name, e.g. `MIT License`.
    pub name: &'a str,
    /// Distinct crates covered by the notices below.
    ///
    /// Sums to more than the number of dependencies across all licences: a crate
    /// licensed `A AND B` — `ring` is `Apache-2.0 AND ISC` — is covered by both
    /// and counted by both.
    pub crates: usize,
    /// The distinct notices, most-used first. Never empty.
    pub notices: Vec<Notice<'a>>,
}

/// One notice and the crates that ship it.
#[derive(Clone, Debug, PartialEq)]
pub struct Notice<'a> {
    /// The licence file, verbatim — copyright line, wrapping and all. Rendered
    /// preformatted; never reflowed, translated or summarised.
    pub text: &'a str,
    /// The crates this notice covers, by name then version. Never empty.
    pub crates: Vec<&'a CrateRef>,
}

impl Notice<'_> {
    /// The distinct crate names this notice covers, for ordering and for the
    /// collapsed row's label.
    ///
    /// Names, not name-and-version, and deduplicated: a graph holding two
    /// versions of one crate under one notice — `hashbrown` is here three times
    /// — would otherwise label the row `hashbrown, hashbrown, hashbrown`. The
    /// versions are not lost; they are in the inventory the page renders below,
    /// and the count beside the label still counts every one of them.
    pub fn crate_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.crates.iter().map(|c| c.name.as_str()).collect();
        // `crates` is sorted by name then version, so equal names are adjacent.
        names.dedup();
        names
    }
}

/// Fold every run of whitespace to a single space, for comparison only.
///
/// Two licence files that differ solely in line wrapping or trailing whitespace
/// are the same notice, and reproducing both would attribute the same copyright
/// holder twice under two texts a reader cannot tell apart. Only the folded form
/// is compared; the notice the page prints is the first file's bytes.
fn whitespace_folded(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One distinct licence text and the crates it covers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LicenseText {
    /// SPDX short identifier this text was identified as.
    pub id: String,
    /// Full name of that licence.
    pub name: String,
    /// The licence file, verbatim.
    pub text: String,
    /// The crates this exact file was found in.
    #[serde(default)]
    pub used_by: Vec<CrateRef>,
}

/// A crate named from a licence text's coverage list.
///
/// Name and version together, because a dependency graph can hold two versions of the same crate
/// under two different copyright lines.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// under. Both are shown: the offer in the inventory, the resolved text in
    /// [`LicensesFile::notices`].
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
            serde_json::from_str(r#"{"texts":[],"crates":[]}"#).expect("valid");
        assert!(parsed.is_empty());
        assert_eq!(parsed, LicensesFile::default());
    }

    /// Absent optional fields must not fail the whole document: a crate that
    /// declares no repository is not rare, and losing the whole inventory over
    /// one would take the page down with it.
    #[test]
    fn optional_crate_fields_default_rather_than_fail() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"texts":[{"id":"MIT","name":"MIT License","text":"Copyright (c) 2020 Someone"}],
                "crates":[{"name":"anyhow","version":"1.0.0","license":"MIT OR Apache-2.0",
                           "source":"registry+https://github.com/rust-lang/crates.io-index"}]}"#,
        )
        .expect("valid");

        assert!(!parsed.is_empty());
        assert!(parsed.texts[0].used_by.is_empty());
        assert_eq!(parsed.crates[0].repository, None);
    }

    /// The workspace's own crates arrive in the document with no source and the
    /// licence `Unknown`; a page about third parties must not list them, and a
    /// document holding nothing else must report itself empty.
    #[test]
    fn path_dependencies_are_not_third_party() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"texts":[],"crates":[
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

    /// An inventory holding nothing but this repository's own crates is as empty
    /// as no inventory at all — otherwise the page would render three headings
    /// over an empty list instead of saying there is nothing to show.
    #[test]
    fn an_inventory_of_only_path_dependencies_is_empty() {
        let parsed: LicensesFile = serde_json::from_str(
            r#"{"texts":[],
                "crates":[{"name":"web","version":"2.6.0","license":"Unknown"}]}"#,
        )
        .expect("valid");

        assert!(parsed.is_empty());
    }

    /// Builds a document the way cargo-about reports one: one entry per licence
    /// *file*, so the same licence recurs once per crate that carries it, and
    /// every crate named is third-party.
    fn document(entries: &[(&str, &str, &str, &[&str])]) -> LicensesFile {
        let mut crates: BTreeSet<&str> = BTreeSet::new();
        for (_, _, _, names) in entries {
            crates.extend(names.iter().copied());
        }

        LicensesFile {
            texts: entries
                .iter()
                .map(|(id, name, text, names)| LicenseText {
                    id: (*id).to_owned(),
                    name: (*name).to_owned(),
                    text: (*text).to_owned(),
                    used_by: names
                        .iter()
                        .map(|n| CrateRef {
                            name: (*n).to_owned(),
                            version: "1.0.0".to_owned(),
                        })
                        .collect(),
                })
                .collect(),
            crates: crates
                .into_iter()
                .map(|name| CrateLicense {
                    name: name.to_owned(),
                    version: "1.0.0".to_owned(),
                    license: "MIT".to_owned(),
                    source: Some("registry+x".to_owned()),
                    repository: None,
                })
                .collect(),
        }
    }

    /// The bug this view exists to fix. The page's first shape rendered a row per
    /// dependency, each reproducing the licence texts covering it, so a notice
    /// shared by 57 crates was served 57 times — 499 KB of licence text for
    /// 215 KB of distinct notices, in a 738 KB document. A notice is reproduced
    /// once, and every crate carrying it is still attributed.
    #[test]
    fn an_identical_notice_is_reproduced_once_however_many_crates_ship_it() {
        const MIT: &str = "MIT License\n\nCopyright (c) 2020 A. Person\n";
        let file = document(&[
            ("MIT", "MIT License", MIT, &["alpha"]),
            ("MIT", "MIT License", MIT, &["beta"]),
            ("MIT", "MIT License", MIT, &["gamma"]),
        ]);

        let licences = file.notices();
        assert_eq!(licences.len(), 1);
        assert_eq!(licences[0].notices.len(), 1, "one text is not three");
        assert_eq!(licences[0].crates, 3);
        assert_eq!(
            licences[0].notices[0].crate_names(),
            ["alpha", "beta", "gamma"],
            "merging must not cost a crate its attribution"
        );
    }

    /// Two copies of one notice that differ only in how they were wrapped are one
    /// notice. Without the fold they are two, and the page reproduces the same
    /// copyright holder twice under texts a reader cannot tell apart.
    #[test]
    fn notices_differing_only_in_whitespace_are_merged_and_the_first_kept_verbatim() {
        let file = document(&[
            (
                "MIT",
                "MIT License",
                "Copyright (c) 2020 A. Person\nMIT\n",
                &["alpha"],
            ),
            (
                "MIT",
                "MIT License",
                "Copyright (c) 2020    A. Person MIT",
                &["beta"],
            ),
        ]);

        let licences = file.notices();
        assert_eq!(licences[0].notices.len(), 1);
        assert_eq!(
            licences[0].notices[0].text, "Copyright (c) 2020 A. Person\nMIT\n",
            "the reproduced notice is the first file's bytes, not a folded form"
        );
    }

    /// A copyright line is the part of an MIT notice the licence requires be
    /// reproduced, so two notices under one identifier stay two.
    #[test]
    fn different_copyright_holders_under_one_identifier_stay_distinct() {
        let file = document(&[
            (
                "MIT",
                "MIT License",
                "Copyright (c) 2020 A. Person",
                &["alpha"],
            ),
            (
                "MIT",
                "MIT License",
                "Copyright (c) 2021 B. Person",
                &["beta"],
            ),
        ]);

        let licences = file.notices();
        assert_eq!(licences[0].notices.len(), 2);
        assert_eq!(licences[0].crates, 2);
    }

    /// A crate shipping several notices — `ring` vendors eighteen — is one crate
    /// under that licence, not eighteen. The chips used to render the sum, which
    /// is why `ISC` claimed 19 dependencies over a graph holding four.
    #[test]
    fn a_crate_shipping_several_notices_is_counted_once() {
        let file = document(&[
            ("ISC", "ISC License", "notice one", &["ring"]),
            ("ISC", "ISC License", "notice two", &["ring"]),
            ("ISC", "ISC License", "notice three", &["untrusted"]),
        ]);

        let licences = file.notices();
        assert_eq!(licences[0].notices.len(), 3);
        assert_eq!(licences[0].crates, 2, "two crates, three notices");
    }

    /// The order the page renders in has to come out of the data, or two builds
    /// of an unchanged graph produce two different pages: licences by coverage
    /// then identifier, notices by coverage then by the crates they name.
    #[test]
    fn licences_and_notices_are_ordered_by_coverage() {
        let file = document(&[
            ("MIT", "MIT License", "mit-rare", &["solo"]),
            (
                "MIT",
                "MIT License",
                "mit-common",
                &["alpha", "beta", "gamma"],
            ),
            ("ISC", "ISC License", "isc", &["delta", "epsilon"]),
            ("Zlib", "zlib License", "zlib", &["zeta", "eta"]),
        ]);

        let licences = file.notices();
        let ids: Vec<&str> = licences.iter().map(|l| l.id).collect();
        assert_eq!(
            ids,
            ["MIT", "ISC", "Zlib"],
            "MIT covers most; ISC and Zlib tie on four crates and break on the identifier"
        );

        let texts: Vec<&str> = licences[0].notices.iter().map(|n| n.text).collect();
        assert_eq!(texts, ["mit-common", "mit-rare"]);
    }

    /// A licence file this repository wrote must not be reproduced as a third
    /// party's, and a notice naming nothing else must not leave an empty section
    /// behind — the same rule `third_party` applies to the inventory.
    #[test]
    fn a_notice_naming_only_path_dependencies_is_dropped() {
        let file = LicensesFile {
            texts: vec![
                LicenseText {
                    id: "LicenseRef-Proprietary".to_owned(),
                    name: "Proprietary".to_owned(),
                    text: "ours".to_owned(),
                    used_by: vec![CrateRef {
                        name: "web".to_owned(),
                        version: "2.7.1".to_owned(),
                    }],
                },
                LicenseText {
                    id: "MIT".to_owned(),
                    name: "MIT License".to_owned(),
                    text: "theirs".to_owned(),
                    used_by: vec![
                        CrateRef {
                            name: "web".to_owned(),
                            version: "2.7.1".to_owned(),
                        },
                        CrateRef {
                            name: "anyhow".to_owned(),
                            version: "1.0.0".to_owned(),
                        },
                    ],
                },
            ],
            crates: vec![
                CrateLicense {
                    name: "web".to_owned(),
                    version: "2.7.1".to_owned(),
                    license: "Unknown".to_owned(),
                    source: None,
                    repository: None,
                },
                CrateLicense {
                    name: "anyhow".to_owned(),
                    version: "1.0.0".to_owned(),
                    license: "MIT".to_owned(),
                    source: Some("registry+x".to_owned()),
                    repository: None,
                },
            ],
        };

        let licences = file.notices();
        let ids: Vec<&str> = licences.iter().map(|l| l.id).collect();
        assert_eq!(ids, ["MIT"]);
        assert_eq!(licences[0].notices[0].crate_names(), ["anyhow"]);
    }

    /// A graph can hold two versions of one crate under one notice, and the label
    /// a collapsed row carries must not read `hashbrown, hashbrown, hashbrown`.
    /// The count beside it still counts every version.
    #[test]
    fn a_notice_labels_a_crate_once_however_many_versions_it_covers() {
        let file = LicensesFile {
            texts: vec![LicenseText {
                id: "MIT".to_owned(),
                name: "MIT License".to_owned(),
                text: "shared".to_owned(),
                used_by: ["0.14.5", "0.15.5", "0.16.0"]
                    .into_iter()
                    .map(|version| CrateRef {
                        name: "hashbrown".to_owned(),
                        version: version.to_owned(),
                    })
                    .collect(),
            }],
            crates: ["0.14.5", "0.15.5", "0.16.0"]
                .into_iter()
                .map(|version| CrateLicense {
                    name: "hashbrown".to_owned(),
                    version: version.to_owned(),
                    license: "MIT".to_owned(),
                    source: Some("registry+x".to_owned()),
                    repository: None,
                })
                .collect(),
        };

        let licences = file.notices();
        assert_eq!(licences[0].notices[0].crate_names(), ["hashbrown"]);
        assert_eq!(licences[0].notices[0].crates.len(), 3);
        assert_eq!(licences[0].crates, 3);
    }

    /// A licence text is reproduced byte for byte; the round-trip is what proves
    /// no serialisation step here rewrites one.
    #[test]
    fn licence_text_survives_a_round_trip_verbatim() {
        let text = "MIT License\n\nCopyright (c) 2020 A. Person <a@example.com>\n\n\"Software\"\n";
        let file = LicensesFile {
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
