//! The skill matrix: quadrants, confidence values and what the matrix and resume show.

/// The confidence below which a skill is left out of the matrix and the resume.
pub const MIN_CONFIDENCE: f32 = 0.6;

/// A region of the tech radar, which is also a group of the skill matrix.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quadrant {
    /// Programming languages, and the markup and configuration formats filed with them
    /// (`Markdown`, `TOML`, `Dockerfile`).
    Languages,
    /// Libraries and frameworks.
    ///
    /// A datastore's client library is filed here and the datastore itself under
    /// [`Infra`](Self::Infra): `sqlx` and `Prisma` against `PostgreSQL`.
    Frameworks,
    /// Build, test and repository tooling.
    ///
    /// The profile API withholds this quadrant; see
    /// [`profile::ProfileSkills`](crate::profile::ProfileSkills).
    Build,
    /// Runtimes, datastores and the deployment machinery around them.
    Infra,
}

impl Quadrant {
    /// Returns the translation key of the quadrant's label.
    #[must_use]
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Quadrant::Languages => "skills.languages",
            Quadrant::Frameworks => "skills.frameworks",
            Quadrant::Build => "skills.build",
            Quadrant::Infra => "skills.infrastructure",
        }
    }

    /// Returns the color the quadrant is drawn in on the skill radar and its legend.
    #[must_use]
    pub fn color(&self) -> &'static str {
        match self {
            Quadrant::Languages => "#60a5fa",
            Quadrant::Frameworks => "#22d3ee",
            Quadrant::Build => "#34d399",
            Quadrant::Infra => "#a78bfa",
        }
    }

    /// Returns every quadrant, in the order the radar draws them.
    #[must_use]
    pub fn all() -> [Quadrant; 4] {
        [
            Quadrant::Languages,
            Quadrant::Frameworks,
            Quadrant::Build,
            Quadrant::Infra,
        ]
    }
}

/// One entry of the skill inventory.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Skill {
    /// The tool's name, shown as written in every language.
    pub name: &'static str,
    /// The radar region the skill is plotted in, which is also its matrix group.
    pub quadrant: Quadrant,
    /// The hands-on depth, from `0.0` to `1.0`.
    ///
    /// It decides the radar distance, the matrix order and, against [`MIN_CONFIDENCE`], whether
    /// the skill is listed at all.
    pub confidence: f32,
    /// Whether the skill appears on the radar alone, never in the matrix or on the resume.
    pub radar_only: bool,
}

impl Skill {
    /// Returns the confidence on a scale of one to five, rounded to the nearest integer.
    ///
    /// The result is clamped to `1..=5`, so a skill nobody would list still gets one star.
    ///
    /// ```
    /// # use portfolio_data::{Quadrant, Skill};
    /// # fn skill(confidence: f32) -> Skill {
    /// #     Skill { name: "x", quadrant: Quadrant::Languages, confidence, radar_only: false }
    /// # }
    /// assert_eq!(skill(0.95).level(), 5);
    /// assert_eq!(skill(0.70).level(), 4);
    /// assert_eq!(skill(0.05).level(), 1);
    /// ```
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "`as` saturates, and the result is clamped to 1..=5 either way"
    )]
    pub fn level(&self) -> u8 {
        ((self.confidence * 5.0).round() as u8).clamp(1, 5)
    }
}

const fn s(name: &'static str, quadrant: Quadrant, confidence: f32) -> Skill {
    Skill {
        name,
        quadrant,
        confidence,
        radar_only: false,
    }
}

const fn r(name: &'static str, quadrant: Quadrant, confidence: f32) -> Skill {
    Skill {
        name,
        quadrant,
        confidence,
        radar_only: true,
    }
}

/// The full skill inventory.
///
/// Matrix skills, written `s(…)`, reach the skill section, the resume and the radar when their
/// confidence is at least [`MIN_CONFIDENCE`]. Radar-only skills, written `r(…)`, reach the radar
/// alone, whatever their confidence.
pub const SKILLS: &[Skill] = &{
    use Quadrant::{Build, Frameworks, Infra, Languages};
    [
        // languages
        s("Java", Languages, 0.95),
        s("Rust", Languages, 0.90),
        s("SQL", Languages, 0.85),
        s("TypeScript", Languages, 0.75),
        s("WebAssembly (WASM)", Languages, 0.75),
        s("JavaScript", Languages, 0.50),
        s("Kotlin", Languages, 0.45),
        s("Python", Languages, 0.45),
        s("C#", Languages, 0.35),
        s("Lua", Languages, 0.25),
        s("Go", Languages, 0.20),
        s("C++", Languages, 0.10),
        r("YAML", Languages, 0.85),
        r("Dockerfile", Languages, 0.80),
        r("Markdown", Languages, 0.70),
        r("Regular Expressions", Languages, 0.70),
        r("Bash / Shell", Languages, 0.65),
        r("HCL", Languages, 0.65),
        r("TOML", Languages, 0.65),
        r("HTML", Languages, 0.50),
        r("Properties / INI", Languages, 0.50),
        r("Typst", Languages, 0.50),
        r("CSS", Languages, 0.45),
        r("PowerShell", Languages, 0.45),
        // frameworks
        s("Spring Boot", Frameworks, 0.86),
        s("Axum", Frameworks, 0.85),
        s("Dioxus", Frameworks, 0.85),
        s("PaperMC", Frameworks, 0.85),
        s("Tokio", Frameworks, 0.85),
        s("gRPC", Frameworks, 0.82),
        s("Next.js", Frameworks, 0.80),
        s("Tailwind CSS", Frameworks, 0.80),
        s("React", Frameworks, 0.76),
        s("Node.js", Frameworks, 0.65),
        r("Bukkit API", Frameworks, 0.95),
        r("Spigot API", Frameworks, 0.95),
        r("Lombok", Frameworks, 0.85),
        r("Serde", Frameworks, 0.85),
        r("tracing", Frameworks, 0.80),
        r("Jackson", Frameworks, 0.75),
        r("next-intl", Frameworks, 0.75),
        r("Zod", Frameworks, 0.72),
        r("reqwest", Frameworks, 0.70),
        r("shadcn/ui", Frameworks, 0.70),
        r("sqlx", Frameworks, 0.70),
        r("Actix Web", Frameworks, 0.65),
        r("JDBI", Frameworks, 0.65),
        r("OpenRewrite", Frameworks, 0.65),
        r("Pino", Frameworks, 0.65),
        r("Tower", Frameworks, 0.65),
        r("Velocity API", Frameworks, 0.65),
        r("clap", Frameworks, 0.60),
        r("Express", Frameworks, 0.60),
        r("Octokit", Frameworks, 0.60),
        r("PacketEvents", Frameworks, 0.60),
        r("Radix UI", Frameworks, 0.60),
        r("Serwist", Frameworks, 0.60),
        r("utoipa", Frameworks, 0.60),
        r("wasm-bindgen", Frameworks, 0.60),
        r("Webhooks", Frameworks, 0.60),
        r("Yew", Frameworks, 0.60),
        r("Drizzle ORM", Frameworks, 0.55),
        r("React Hook Form", Frameworks, 0.55),
        r("React Query", Frameworks, 0.55),
        r("React-PDF", Frameworks, 0.55),
        r("JavaFX", Frameworks, 0.50),
        r("Prisma", Frameworks, 0.50),
        r("ratatui", Frameworks, 0.50),
        r("NextAuth.js", Frameworks, 0.45),
        r("tRPC", Frameworks, 0.40),
        r("Rocket", Frameworks, 0.30),
        // build & tools
        s("GitHub Actions", Build, 0.95),
        s("Git", Build, 0.90),
        s("Gradle", Build, 0.90),
        s("Cargo", Build, 0.85),
        s("JUnit", Build, 0.85),
        s("Mockito", Build, 0.85),
        s("just", Build, 0.80),
        s("Maven", Build, 0.80),
        s("Playwright", Build, 0.70),
        r("release-please", Build, 0.90),
        r("Renovate", Build, 0.90),
        r("AssertJ", Build, 0.85),
        r("Checkstyle", Build, 0.85),
        r("Clippy", Build, 0.85),
        r("Bun", Build, 0.80),
        r("cargo-deny", Build, 0.80),
        r("ESLint", Build, 0.80),
        r("Prettier", Build, 0.80),
        r("Zizmor", Build, 0.80),
        r("cargo-nextest", Build, 0.75),
        r("Codecov", Build, 0.75),
        r("CodeQL", Build, 0.75),
        r("Cosign", Build, 0.75),
        r("Docker Buildx", Build, 0.75),
        r("Harden-Runner", Build, 0.75),
        r("SonarQube", Build, 0.75),
        r("Spotless", Build, 0.75),
        r("Flyway", Build, 0.70),
        r("PMD", Build, 0.70),
        r("SpotBugs", Build, 0.70),
        r("Testcontainers", Build, 0.70),
        r("Trivy", Build, 0.70),
        r("cargo-fuzz", Build, 0.65),
        r("JaCoCo", Build, 0.65),
        r("pre-commit", Build, 0.65),
        r("cargo-mutants", Build, 0.60),
        r("Husky", Build, 0.60),
        r("Jest", Build, 0.60),
        r("npm", Build, 0.60),
        r("pnpm", Build, 0.60),
        r("proptest", Build, 0.60),
        r("Vitest", Build, 0.60),
        r("container-structure-test", Build, 0.55),
        r("lint-staged", Build, 0.55),
        r("ORAS", Build, 0.55),
        r("Storybook", Build, 0.55),
        r("Syft", Build, 0.55),
        r("Testing Library", Build, 0.55),
        r("commitlint", Build, 0.50),
        r("Hadolint", Build, 0.50),
        r("Knip", Build, 0.50),
        // infrastructure
        s("Docker", Infra, 0.90),
        s("Helm", Infra, 0.90),
        s("ArgoCD", Infra, 0.85),
        s("Kubernetes", Infra, 0.85),
        s("PostgreSQL", Infra, 0.80),
        s("Talos Linux", Infra, 0.80),
        s("Linux", Infra, 0.75),
        s("TimescaleDB", Infra, 0.75),
        s("OpenTofu", Infra, 0.70),
        s("Redis", Infra, 0.65),
        s("MongoDB", Infra, 0.50),
        r("Cert-Manager", Infra, 0.80),
        r("Cilium", Infra, 0.80),
        r("Docker Compose", Infra, 0.80),
        r("Sentry", Infra, 0.80),
        r("CloudNativePG", Infra, 0.75),
        r("Cloudflare Tunnels", Infra, 0.75),
        r("External Secrets Operator", Infra, 0.75),
        r("ExternalDNS", Infra, 0.75),
        r("Grafana", Infra, 0.75),
        r("Prometheus", Infra, 0.75),
        r("Sealed Secrets", Infra, 0.75),
        r("VictoriaMetrics", Infra, 0.75),
        r("Bitwarden Secrets Manager", Infra, 0.70),
        r("Gateway API", Infra, 0.70),
        r("k0s", Infra, 0.70),
        r("K8up", Infra, 0.70),
        r("AWS S3", Infra, 0.65),
        r("CrowdSec", Infra, 0.65),
        r("MariaDB", Infra, 0.65),
        r("MinIO", Infra, 0.65),
        r("MySQL", Infra, 0.65),
        r("NATS", Infra, 0.65),
        r("OpenTelemetry", Infra, 0.65),
        r("SQLite", Infra, 0.65),
        r("TopoLVM", Infra, 0.65),
        r("Harbor", Infra, 0.60),
        r("Loki", Infra, 0.60),
        r("MetalLB", Infra, 0.60),
        r("Tempo", Infra, 0.60),
        r("Traefik", Infra, 0.60),
        r("OpenEBS", Infra, 0.55),
        r("Oracle Cloud (OCI)", Infra, 0.55),
        r("Docker Hub", Infra, 0.50),
        r("Elasticsearch", Infra, 0.50),
        r("Nginx", Infra, 0.50),
        r("Apache Kafka", Infra, 0.45),
        r("Cloudflare Workers", Infra, 0.45),
        r("Reverse Proxies (general)", Infra, 0.45),
        r("RabbitMQ", Infra, 0.35),
        r("Pingora", Infra, 0.30),
    ]
};

/// Returns the skills the matrix and the resume show, strongest first.
///
/// These are the skills that are not radar-only and reach [`MIN_CONFIDENCE`].
#[must_use]
pub fn matrix_skills() -> Vec<Skill> {
    let mut out: Vec<Skill> = SKILLS
        .iter()
        .filter(|s| !s.radar_only && s.confidence >= MIN_CONFIDENCE)
        .copied()
        .collect();
    out.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}
