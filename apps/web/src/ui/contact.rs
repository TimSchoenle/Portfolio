//! Contact section: a terminal block, the oversized email link, action buttons,
//! and the language-specific resume PDF with its SHA-256 fingerprint.
//!
//! The type-in animation, clipboard copy and scroll-trigger are client-only
//! enhancements re-added in the hydration phase; here the command renders in
//! full and the copy button gives immediate visual feedback.

use dioxus::prelude::*;
use portfolio_data::{CONFIG, resume_file};

use crate::github::load_resume_fingerprints;
use crate::i18n::use_i18n;
use crate::sections::{section_id, section_label};
use crate::ui::reveal::Reveal;
use crate::ui::section_header::SectionHeader;

#[component]
pub fn Contact() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);
    let mut copied = use_signal(|| false);
    let mut show_fingerprint = use_signal(|| false);
    let fingerprints = load_resume_fingerprints();

    let lang = i18n.read().get_current_language().to_string();
    let resume_name = resume_file(&lang);
    let resume_label = if lang == "de" {
        t("contact.resumeDe")
    } else {
        t("contact.resumeEn")
    };
    let resume_digest = fingerprints.as_ref().and_then(|f| {
        f.digest_for(&lang)
            .map(|d| (f.algorithm.clone(), d.to_string()))
    });

    let mailto = format!("mailto:{}", CONFIG.email);
    let url_display = CONFIG
        .url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let typed = format!("ssh {}", CONFIG.email);
    let copy_label = if copied() {
        t("contact.copied")
    } else {
        t("contact.copy")
    };

    rsx! {
        section { id: section_id("contact"), class: "sec",
            Reveal {
                SectionHeader {
                    num: section_label("contact"),
                    title: t("contact.title"),
                    intro: t("contact.intro"),
                }
            }

            div { class: "grid-12",
                div { class: "col-label" }
                div { class: "col-body",
                    Reveal { delay: 120,
                        div { class: "terminal-block",
                            div { class: "terminal-head",
                                span { class: "term-dots",
                                    span {}
                                    span {}
                                    span {}
                                }
                                span { class: "mono text-muted", "~/contact — zsh" }
                                span {}
                            }
                            div { class: "terminal-body",
                                div { class: "term-line",
                                    span { class: "term-prompt", "$" }
                                    span { class: "term-cmd",
                                        "{typed}"
                                        span { class: "term-cursor", "▊" }
                                    }
                                }
                                div { class: "term-output",
                                    span { class: "mono text-muted", "connecting..." }
                                    span { class: "mono text-accent ml-3", "ok" }
                                }
                            }
                        }
                    }

                    Reveal { delay: 200,
                        div { class: "contact-email-display",
                            span { class: "email-prefix text-accent", ">_" }
                            a { href: "{mailto}", class: "email-link", "{CONFIG.email}" }
                        }
                    }

                    Reveal { delay: 260,
                        div { class: "contact-actions",
                            button {
                                class: "btn-accent",
                                onclick: move |_| {
                                    #[cfg(feature = "web")]
                                    if let Some(win) = web_sys::window() {
                                        let _ = win.navigator().clipboard().write_text(CONFIG.email);
                                    }
                                    copied.set(true);
                                },
                                span { class: "mono", "{copy_label}" }
                            }
                            a { href: "{mailto}", class: "btn-outline",
                                span { class: "mono", {format!("{} →", t("contact.write"))} }
                            }
                            a { href: CONFIG.github, target: "_blank", rel: "noreferrer", class: "btn-outline",
                                span { class: "mono", "GitHub →" }
                            }
                            a { href: CONFIG.linkedin, target: "_blank", rel: "noreferrer", class: "btn-outline",
                                span { class: "mono", "LinkedIn →" }
                            }
                            div { class: "resume-btn-group",
                                a {
                                    href: "/resume/{resume_name}",
                                    target: "_blank",
                                    rel: "noopener",
                                    class: "btn-outline resume-dl",
                                    span { class: "mono", "{resume_label} ↓" }
                                }
                                if let Some((algorithm, digest)) = resume_digest {
                                    button {
                                        r#type: "button",
                                        class: "resume-info-btn",
                                        "aria-label": "{algorithm} fingerprint",
                                        title: "{algorithm} fingerprint",
                                        onclick: move |e| {
                                            e.prevent_default();
                                            e.stop_propagation();
                                            let cur = show_fingerprint();
                                            show_fingerprint.set(!cur);
                                        },
                                        span { class: "mono", "ⓘ" }
                                    }
                                    if show_fingerprint() {
                                        div { class: "fp-popup",
                                            span { class: "mono text-muted", "{algorithm} fingerprint" }
                                            div { class: "fp-row", title: "{digest}",
                                                span { class: "text-fg/80 shrink-0", "{resume_name}" }
                                                span { class: "fp-digest", "{digest}" }
                                            }
                                            span { class: "text-muted text-[11.5px]", {t("contact.fingerprintNote")} }
                                        }
                                    }
                                }
                            }
                            a { href: CONFIG.url, target: "_blank", rel: "noreferrer", class: "btn-outline",
                                span { class: "mono", "{url_display} ↗" }
                            }
                        }
                    }
                }
            }
        }
    }
}
