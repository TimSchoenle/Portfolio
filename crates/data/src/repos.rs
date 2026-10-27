//! The `repos.json` schema `portfolio-update-repos` writes and the web binary reads.

use serde::{Deserialize, Serialize};

use crate::CONFIG;

/// One repository, as `portfolio-update-repos` read it out of the GitHub REST API.
///
/// It deserializes GitHub's own response, so the field names are GitHub's, and every field but
/// three takes its default when GitHub omits it. The same type is written to `repos.json` and read
/// back by the web binary, so the writer and the reader cannot disagree on the document.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Repo {
    /// The repository name without the owner, such as `Portfolio`.
    pub name: String,
    /// The repository name with the owner, as `owner/name`.
    #[serde(default)]
    pub full_name: String,
    /// GitHub's description, or `None` when the repository declares none.
    pub description: Option<String>,
    /// The repository page the card links to.
    pub html_url: String,
    /// GitHub's primary-language guess, or `None` for an empty repository.
    ///
    /// [`lang_color`] colors it.
    #[serde(default)]
    pub language: Option<String>,
    /// The star count when the listing was taken.
    #[serde(default)]
    pub stargazers_count: u32,
    /// The fork count when the listing was taken.
    #[serde(default)]
    pub forks_count: u32,
    /// GitHub's `updated_at`, an RFC 3339 timestamp of the last change to the repository object.
    ///
    /// A push, a description edit and a topic change each move it.
    #[serde(default)]
    pub updated_at: String,
    /// The GitHub topics, rendered as chips.
    #[serde(default)]
    pub topics: Vec<String>,
    /// Whether the repository is a fork.
    ///
    /// Nothing filters on it, so a fork in a generated listing was meant to be there.
    #[serde(default)]
    pub fork: bool,
    /// Whether GitHub has archived the repository.
    ///
    /// Always `false` in an account listing, which drops archived repositories. A repository
    /// named in `github.repos` is written through unfiltered.
    #[serde(default)]
    pub archived: bool,
    /// The site the repository declares, linked beside the source link.
    #[serde(default)]
    pub homepage: Option<String>,
}

impl Repo {
    /// Reports whether [`CONFIG`] pins the repository to the front of the projects section.
    ///
    /// The name is matched case-insensitively.
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
    /// The RFC 3339 timestamp the listing was taken at.
    ///
    /// `portfolio-update-repos` reads it back on the next build to decide whether the file is
    /// fresh enough to skip the API call.
    pub generated_at: String,
    /// The account the repositories were listed for.
    pub user: String,
    /// The repositories, most recently updated first.
    ///
    /// When `github.repos` named them, they are in the configured order instead.
    pub repos: Vec<Repo>,
}

/// Returns the color GitHub paints `lang` in, for the dot on a repository card.
///
/// The values are linguist's, so a card matches what a visitor sees on github.com. A language not
/// in the table gets a neutral grey.
///
/// ```
/// # use portfolio_data::lang_color;
/// assert_eq!(lang_color("Rust"), "#dea584");
/// assert_eq!(lang_color("Brainfuck"), "#9aa4b2");
/// ```
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
