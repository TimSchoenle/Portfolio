//! Public profile API model, mirroring the original `models/api.ts`.
//!
//! The document is assembled from [`crate::CONFIG`] and [`crate::SKILLS`] and
//! serialized by the server's `/api/v1/profile` route. It is language-neutral
//! and stable, so the server builds it once and caches it.
//!
//! The JSON Schema served at `/api/v1/profile/schema` is derived from these
//! types via `schemars`; that derive is behind the `schema` feature so the
//! WebAssembly frontend, which only reads the data, does not pull it in.

use serde::Serialize;

use crate::{CONFIG, Quadrant, SKILLS, Skill};

/// Path of the profile JSON Schema, relative to [`CONFIG`]`.url`.
pub const SCHEMA_PATH: &str = "/api/v1/profile/schema";

/// Where a skill is surfaced across the site. Mirrors `SKILL_RENDER_AREAS`
/// from the original `types/skill.ts`.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum RenderArea {
    Resume,
    Section,
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

/// A skill as exposed by the API. Mirrors `SkillWithConfidence`.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProfileSkill {
    pub confidence: f32,
    pub name: &'static str,
    pub render_area: Vec<RenderArea>,
}

/// Skills grouped by category. The `Build` quadrant is intentionally omitted:
/// the API exposes only languages, frameworks and infrastructure.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProfileSkills {
    pub frameworks: Vec<ProfileSkill>,
    pub infrastructure: Vec<ProfileSkill>,
    pub languages: Vec<ProfileSkill>,
}

#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProfileSocials {
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub github: &'static str,
    pub github_username: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub linkedin: Option<&'static str>,
}

/// The public profile document. Mirrors `profileApiSchema` (the `$schema`
/// pointer is added separately, see [`ProfileWithSchema`]).
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[cfg_attr(feature = "schema", schemars(extend("format" = "email")))]
    pub email: &'static str,
    pub full_name: &'static str,
    pub location: &'static str,
    pub name: &'static str,
    pub skills: ProfileSkills,
    pub socials: ProfileSocials,
    pub title: &'static str,
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub website: &'static str,
}

/// A [`Profile`] with the `$schema` pointer appended last, as served by the
/// profile route. Mirrors `ProfileApiWithSchemaResponse`.
#[derive(Serialize)]
pub struct ProfileWithSchema {
    #[serde(flatten)]
    pub profile: Profile,
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
