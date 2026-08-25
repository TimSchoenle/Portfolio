//! The document `/api/v1/profile` serves, and the schema that describes it.
//!
//! Assembled from [`crate::CONFIG`] and [`crate::SKILLS`], language-neutral, and the same bytes
//! on every request, which is why the server builds it once and caches it.
//!
//! The field names and the JSON shape are older than this crate. They were fixed by the Next.js
//! site this one replaced, and the endpoint kept its contract across that rewrite, so a change
//! to any `#[serde(rename)]`, to a field name, or to the `Build` omission in [`ProfileSkills`]
//! is a breaking API change rather than a refactor. The published schema is what a consumer
//! would notice it with.
//!
//! That schema, served at [`SCHEMA_PATH`], is derived from these types by `schemars` under the
//! `schema` feature. The feature is off for the WebAssembly frontend, which only reads the data
//! and would otherwise carry a schema generator into the bundle.

use serde::Serialize;

use crate::{CONFIG, Quadrant, SKILLS, Skill};

/// Path of the profile JSON Schema, relative to [`CONFIG`]`.url`.
pub const SCHEMA_PATH: &str = "/api/v1/profile/schema";

/// One of the places a skill is shown.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum RenderArea {
    /// The generated resume PDF, which lists the strongest matrix skills.
    Resume,
    /// The skills section of the home page.
    Section,
    /// The radar, which is the only area a `radar_only` skill reaches.
    TechRadar,
}

impl RenderArea {
    /// Every area, in render order — the default for a skill that is not
    /// radar-only.
    pub const ALL: [RenderArea; 3] = [
        RenderArea::Resume,
        RenderArea::Section,
        RenderArea::TechRadar,
    ];
}

impl Skill {
    /// The areas a skill is rendered in. Radar-only skills appear solely on
    /// the tech radar; every other skill appears everywhere.
    pub fn render_areas(&self) -> Vec<RenderArea> {
        if self.radar_only {
            vec![RenderArea::TechRadar]
        } else {
            RenderArea::ALL.to_vec()
        }
    }
}

/// A skill as the API publishes it.
///
/// [`crate::Skill`] minus the quadrant, which the grouping in [`ProfileSkills`] already carries,
/// and minus `radar_only`, which [`Self::render_area`] states positively instead.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProfileSkill {
    /// `0.0..=1.0`. Published unrounded and unfiltered, so a consumer can set its own floor
    /// rather than inheriting [`crate::MIN_CONFIDENCE`].
    pub confidence: f32,
    /// The tool's name, untranslated.
    pub name: &'static str,
    /// Every area this skill appears in.
    pub render_area: Vec<RenderArea>,
}

/// Skills grouped by quadrant, three of the four.
///
/// [`Quadrant::Build`] has no field here, and the omission is part of the published contract.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProfileSkills {
    /// Every [`Quadrant::Frameworks`] skill, in inventory order.
    pub frameworks: Vec<ProfileSkill>,
    /// Every [`Quadrant::Infra`] skill, in inventory order.
    pub infrastructure: Vec<ProfileSkill>,
    /// Every [`Quadrant::Languages`] skill, in inventory order.
    pub languages: Vec<ProfileSkill>,
}

/// Where else to find the person the profile describes.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProfileSocials {
    /// GitHub profile URL.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub github: &'static str,
    /// The account name on its own, so a consumer can build an API call without parsing
    /// [`Self::github`].
    pub github_username: &'static str,
    /// LinkedIn profile URL. Omitted from the JSON entirely when absent, rather than serialized
    /// as `null`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub linkedin: Option<&'static str>,
}

/// The profile document itself, without the `$schema` pointer.
///
/// Declared alphabetically, which is the order `serde` emits them in, so reordering the fields
/// reorders the response.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// Public contact address.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "email")))]
    pub email: &'static str,
    /// Name in full.
    pub full_name: &'static str,
    /// Country, not a city or an address.
    pub location: &'static str,
    /// Given name alone.
    pub name: &'static str,
    /// The published skill set.
    pub skills: ProfileSkills,
    /// Profile links.
    pub socials: ProfileSocials,
    /// The role on its own: [`crate::Config::job_title`], not [`crate::Config::title`].
    pub title: &'static str,
    /// The site's canonical origin.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub website: &'static str,
}

/// What the route actually sends: a [`Profile`] with its `$schema` pointer.
///
/// The pointer is appended rather than declared first, so it is the last key in the object. That
/// is cosmetic for a parser and deliberate for a human reading the raw response, who wants the
/// profile before the metadata about it.
#[derive(Serialize)]
pub struct ProfileWithSchema {
    /// Flattened into the top-level object, so the wire format has no nesting here.
    #[serde(flatten)]
    pub profile: Profile,
    /// Absolute URL of the schema this document validates against, built from
    /// [`crate::Config::url`] and [`SCHEMA_PATH`].
    #[serde(rename = "$schema")]
    pub schema: String,
}

/// Builds the public profile document from the compile-time configuration.
pub fn profile() -> ProfileWithSchema {
    ProfileWithSchema {
        profile: Profile {
            email: CONFIG.email,
            full_name: CONFIG.full_name,
            location: CONFIG.location,
            name: CONFIG.name,
            skills: ProfileSkills {
                frameworks: skills_in(Quadrant::Frameworks),
                infrastructure: skills_in(Quadrant::Infra),
                languages: skills_in(Quadrant::Languages),
            },
            socials: ProfileSocials {
                github: CONFIG.github,
                github_username: CONFIG.github_username,
                linkedin: Some(CONFIG.linkedin),
            },
            title: CONFIG.job_title,
            website: CONFIG.url,
        },
        schema: format!("{}{SCHEMA_PATH}", CONFIG.url),
    }
}

fn skills_in(quadrant: Quadrant) -> Vec<ProfileSkill> {
    SKILLS
        .iter()
        .filter(|skill| skill.quadrant == quadrant)
        .map(|skill| ProfileSkill {
            confidence: skill.confidence,
            name: skill.name,
            render_area: skill.render_areas(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn document() -> Value {
        serde_json::to_value(profile()).expect("profile serializes")
    }

    #[test]
    fn fields_come_from_config() {
        let doc = document();
        assert_eq!(doc["email"], CONFIG.email);
        assert_eq!(doc["fullName"], CONFIG.full_name);
        assert_eq!(doc["name"], CONFIG.name);
        assert_eq!(doc["location"], CONFIG.location);
        assert_eq!(doc["title"], CONFIG.job_title);
        assert_eq!(doc["website"], CONFIG.url);
        assert_eq!(doc["socials"]["github"], CONFIG.github);
        assert_eq!(doc["socials"]["githubUsername"], CONFIG.github_username);
        assert_eq!(doc["socials"]["linkedin"], CONFIG.linkedin);
        assert_eq!(doc["$schema"], format!("{}{SCHEMA_PATH}", CONFIG.url));
    }

    #[test]
    fn skills_are_grouped_without_build_tools() {
        let doc = document();
        for category in ["frameworks", "infrastructure", "languages"] {
            assert!(
                !doc["skills"][category].as_array().unwrap().is_empty(),
                "no skills in {category}"
            );
        }
        assert!(doc["skills"].get("build").is_none());
        assert!(doc["skills"].get("buildTools").is_none());

        // A `Build`-quadrant skill (e.g. Gradle) must not surface anywhere.
        let exposed = |quadrant: Quadrant| SKILLS.iter().any(|s| s.quadrant == quadrant);
        assert!(exposed(Quadrant::Build), "test premise: Build skills exist");
        for category in ["frameworks", "infrastructure", "languages"] {
            for skill in doc["skills"][category].as_array().unwrap() {
                assert_ne!(skill["name"], "Gradle");
            }
        }
    }

    #[test]
    fn render_areas_reflect_radar_only_flag() {
        let doc = document();
        let languages = doc["skills"]["languages"].as_array().unwrap();
        let render_area = |name: &str| {
            languages
                .iter()
                .find(|skill| skill["name"] == name)
                .unwrap_or_else(|| panic!("missing skill {name}"))["renderArea"]
                .clone()
        };

        // `Java` is a matrix skill -> rendered everywhere.
        assert_eq!(
            render_area("Java"),
            json!(["resume", "section", "tech-radar"])
        );
        // `Markdown` is radar-only -> only the tech radar.
        assert_eq!(render_area("Markdown"), json!(["tech-radar"]));
    }
}
