//! Check a built image's labels against the contract it should be carrying.
//!
//! ```text
//! docker inspect -f '{{json .Config.Labels}}' "$image" \
//!   | cargo run -p portfolio-config --features config-schema --example verify-labels
//! ```
//!
//! This is the step the `LABEL` block in the `Dockerfile` needs and a source diff cannot give.
//! The three values are constants, so they are written out by hand there — which means the ways
//! they go wrong are all invisible to the repository: a `LABEL` line dropped on a branch nobody
//! diffed, a base image contributing its own, a build that produced a different path than the one
//! the block claims.
//!
//! Reads the labels as `docker inspect` and `crane config` report them — a JSON object under
//! `Config.Labels` — so nothing here has to know which of the two produced them.

use std::collections::BTreeMap;
use std::error::Error;
use std::io::Read as _;
use std::process::ExitCode;

use portfolio_data::LANGUAGES;
use terrace_config::schema::DEFAULT_PATH;

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!("the image's labels match the contract");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    // `docker inspect` prints `null` for an image with no labels at all, which is a different
    // failure from an image with the wrong ones and reads better said here.
    let labels: BTreeMap<String, String> = match input.trim() {
        "" | "null" => BTreeMap::new(),
        text => serde_json::from_str(text)?,
    };

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PATH.to_owned());
    // The same contract `--format contract` publishes, built by the same function, so this checks
    // the image against the document the build put in it rather than against a second description
    // of it.
    portfolio_config::schema::contract(&LANGUAGES)?.verify_labels(&path, &labels)?;
    Ok(())
}
