//! The `repos.json` schema `update-repos` writes and the web binary reads.

use serde::{Deserialize, Serialize};

use crate::CONFIG;

/// One repository, as `update-repos` read it out of the GitHub REST API.
///
/// This deserializes GitHub's own response, which is why the field names are GitHub's and why
/// all but three default rather than failing the listing. The same type is then written to
/// `repos.json` and read back by the web binary, so the projects section and the API response
/// cannot describe different documents.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Repo {
    /// Repository name without the owner, e.g. `Portfolio`.
    pub name: String,
    /// `owner/name`.
    #[serde(default)]
    pub full_name: String,
    /// GitHub's own description. `None` for a repository that declares none, and the card then
    /// renders without a summary line.
    pub description: Option<String>,
    /// The repository page the card links to.
    pub html_url: String,
    /// GitHub's primary-language guess, colored by [`lang_color`]. `None` for an empty
    /// repository.
    #[serde(default)]
    pub language: Option<String>,
    /// Stars when the listing was taken, not live.
    #[serde(default)]
    pub stargazers_count: u32,
    /// Forks at that same moment.
    #[serde(default)]
    pub forks_count: u32,
    /// RFC 3339 timestamp of GitHub's `updated_at`: the last change to the repository object,
    /// which a push, a description edit or a topic change each move. `update-repos` drops
    /// anything older than a year.
    #[serde(default)]
    pub updated_at: String,
    /// GitHub topics, rendered as chips.
    #[serde(default)]
    pub topics: Vec<String>,
    /// Whether the repository is a fork. Carried but never filtered on, so a fork in a generated
    /// listing is one that was meant to be there.
    #[serde(default)]
    pub fork: bool,
    /// Whether GitHub has archived it. `false` for every repository an account listing produced,
    /// since that is what the filtering removes; a repository named in `github.repos` is written
    /// through unfiltered.
    #[serde(default)]
    pub archived: bool,
    /// The site the repository declares, linked beside the source link.
    #[serde(default)]
    pub homepage: Option<String>,
}

impl Repo {
    /// Whether [`CONFIG`] pins this repository to the front of the projects section. Matched
    /// case-insensitively by name.
    #[must_use]
    pub fn is_featured(&self) -> bool {
        CONFIG
            .featured_repos
            .iter()
            .any(|f| f.eq_ignore_ascii_case(&self.name))
    }
}

/// The generated `repos.json`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReposFile {
    /// RFC 3339 timestamp the listing was taken at. `update-repos` reads it back on the next
    /// build to decide whether the file is fresh enough to skip the API call.
    pub generated_at: String,
    /// The account the repositories were listed for.
    pub user: String,
    /// The repositories. Most recently updated first when the account was listed, and in the
    /// configured order when `github.repos` named them.
    pub repos: Vec<Repo>,
}

/// The color GitHub paints a language in, for the dot on a repository card.
///
/// The values are linguist's, copied rather than fetched, so a card matches what a visitor sees
/// on github.com. An unrecognised language gets a neutral grey rather than no dot.
#[must_use]
pub fn lang_color(lang: &str) -> &'static str {
    match lang {
        "Rust" => "#dea584",
        "Java" => "#b07219",
        "TypeScript" => "#3178c6",
        "JavaScript" => "#f1e05a",
        "Python" => "#3572A5",
        "Shell" => "#89e051",
        "Go" => "#00ADD8",
        "HTML" => "#e34c26",
        "CSS" => "#563d7c",
        "Dockerfile" => "#384d54",
        "PLpgSQL" => "#336790",
        "Helm" => "#0f1689",
        "Kotlin" => "#A97BFF",
        "C#" => "#178600",
        "C++" => "#f34b7d",
        "Lua" => "#000080",
        "YAML" => "#cb171e",
        "Smarty" => "#f0c040",
        _ => "#9aa4b2",
    }
}
