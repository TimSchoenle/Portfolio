//! Error model for the `update-repos` builder.
//!
//! A single [`UpdateReposError`] enum covers every failure mode of fetching the
//! GitHub repositories and writing `repos.json`, so the binary's `main` can
//! report a precise, human-readable cause and exit non-zero without leaking
//! panics or `Box<dyn Error>` opaqueness through the public builder API.

use std::error::Error;
use std::fmt;

/// Everything that can go wrong while building `repos.json`.
#[derive(Debug)]
pub enum UpdateReposError {
    /// The layered configuration could not be loaded: a value failed to parse,
    /// a mounted file could not be read, or one key was supplied twice.
    Config(portfolio_config::ConfigError),
    /// The GitHub username to query was empty.
    MissingUser,
    /// No repositories were found to write (the user has no non-archived
    /// repositories, or the explicitly configured set was empty).
    NoRepos,
    /// An HTTP transport failure or a non-2xx response from the GitHub API
    /// (`ureq` surfaces HTTP status codes as errors by default).
    Http(Box<ureq::Error>),
    /// Reading or writing a file on disk failed.
    Io(std::io::Error),
    /// Serializing the assembled [`portfolio_data::ReposFile`] to JSON failed.
    Serialize(serde_json::Error),
    /// Formatting the `generated_at` RFC 3339 timestamp failed.
    Timestamp(time::error::Format),
}

impl fmt::Display for UpdateReposError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateReposError::Config(e) => write!(f, "configuration is not usable: {e}"),
            UpdateReposError::MissingUser => f.write_str(
                "no GitHub username configured (set PORTFOLIO_GITHUB__USERNAME, or `github.username`)",
            ),
            UpdateReposError::NoRepos => f.write_str(
                "no repositories found to write (none active, or `github.repos` named none)",
            ),
            UpdateReposError::Http(e) => write!(f, "GitHub API request failed: {e}"),
            UpdateReposError::Io(e) => write!(f, "file I/O failed: {e}"),
            UpdateReposError::Serialize(e) => write!(f, "serializing repos.json failed: {e}"),
            UpdateReposError::Timestamp(e) => write!(f, "formatting timestamp failed: {e}"),
        }
    }
}

impl Error for UpdateReposError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UpdateReposError::Config(e) => Some(e),
            UpdateReposError::MissingUser => None,
            UpdateReposError::NoRepos => None,
            UpdateReposError::Http(e) => Some(e),
            UpdateReposError::Io(e) => Some(e),
            UpdateReposError::Serialize(e) => Some(e),
            UpdateReposError::Timestamp(e) => Some(e),
        }
    }
}

impl From<portfolio_config::ConfigError> for UpdateReposError {
    fn from(e: portfolio_config::ConfigError) -> Self {
        UpdateReposError::Config(e)
    }
}

impl From<ureq::Error> for UpdateReposError {
    fn from(e: ureq::Error) -> Self {
        UpdateReposError::Http(Box::new(e))
    }
}

impl From<std::io::Error> for UpdateReposError {
    fn from(e: std::io::Error) -> Self {
        UpdateReposError::Io(e)
    }
}

impl From<serde_json::Error> for UpdateReposError {
    fn from(e: serde_json::Error) -> Self {
        UpdateReposError::Serialize(e)
    }
}

impl From<time::error::Format> for UpdateReposError {
    fn from(e: time::error::Format) -> Self {
        UpdateReposError::Timestamp(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, ErrorKind};

    /// The message has to name the key an operator would actually set, in the
    /// dialect the loader speaks — the whole point of the migration is that
    /// there is one spelling, so an error naming a retired variable would send
    /// them to a variable nothing reads.
    #[test]
    fn missing_user_has_actionable_message() {
        let msg = UpdateReposError::MissingUser.to_string();
        assert!(msg.contains("PORTFOLIO_GITHUB__USERNAME"), "{msg}");
        assert!(UpdateReposError::MissingUser.source().is_none());
    }

    #[test]
    fn io_error_is_wrapped_and_chained() {
        let err: UpdateReposError = io::Error::new(ErrorKind::NotFound, "nope").into();
        assert!(matches!(err, UpdateReposError::Io(_)));
        assert!(err.to_string().starts_with("file I/O failed"));
        assert!(err.source().is_some(), "wrapped error exposes its cause");
    }

    #[test]
    fn serialize_error_is_wrapped() {
        let json_err = serde_json::from_str::<u8>("\"not a number\"").unwrap_err();
        let err: UpdateReposError = json_err.into();
        assert!(matches!(err, UpdateReposError::Serialize(_)));
        assert!(err.to_string().starts_with("serializing repos.json failed"));
    }
}
