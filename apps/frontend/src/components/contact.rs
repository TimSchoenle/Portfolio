//! Contact section from the v4 design: terminal with a type-in `ssh` command,
//! oversized email link, action buttons — plus the resume PDF for the active
//! language with its SHA-256 fingerprint.

use i18nrs::yew::use_translation;
use portfolio_data::{CONFIG, resume_file};
use wasm_bindgen_futures::JsFuture;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::github::load_resume_fingerprints;
use crate::hooks::use_in_view;

use super::reveal::Reveal;
use super::section_header::SectionHeader;
use super::sections::{section_id, section_label};

#[function_component(Contact)]
pub fn contact() -> Html {
    let (i18n, _) = use_translation();
    let copied = use_state(|| false);
    let show_fingerprint = use_state(|| false);
    let typed = use_state(String::new);
    // Embedded at build time; `None` in dev builds without generated resumes.
    let fingerprints = load_resume_fingerprints();
    let section = use_node_ref();

    // Only the resume for the active language is offered and verified.
    let lang = i18n.get_current_language();
    let resume_name = resume_file(lang);
    let resume_label = if lang == "de" {
        i18n.t("contact.resumeDe")
    } else {
        i18n.t("contact.resumeEn")
    };
    let resume_digest = fingerprints.as_ref().and_then(|f| {
        f.digest_for(lang)
            .map(|d| (f.algorithm.clone(), d.to_string()))
    });
    // Sigstore signature for the active language's resume, when the PDF was
    // signed on CI (keyless OIDC). `None` on unsigned dev builds.
    let resume_signature = fingerprints
        .as_ref()
        .and_then(|f| f.signature_for(lang).cloned());

    // Type the `ssh` command once the section scrolls into view.
    let in_view = use_in_view(&section, 0.3);
    {
        let typed = typed.clone();
        use_effect_with(in_view, move |&in_view| {
            if in_view {
                spawn_local(async move {
                    let target: Vec<char> = format!("ssh {}", CONFIG.email).chars().collect();
                    for i in 0..=target.len() {
                        typed.set(target[..i].iter().collect());
                        let jitter = (js_sys::Math::random() * 40.0) as u32;
                        gloo_timers::future::TimeoutFuture::new(40 + jitter).await;
                    }
                });
            }
        });
    }

    let copy_email = {
        let copied = copied.clone();
        Callback::from(move |_| {
            let copied = copied.clone();
            spawn_local(async move {
                if let Some(win) = web_sys::window() {
                    let _ =
                        JsFuture::from(win.navigator().clipboard().write_text(CONFIG.email)).await;
                    copied.set(true);
                    gloo_timers::future::TimeoutFuture::new(1600).await;
                    copied.set(false);
                }
            });
        })
    };

    let toggle_fingerprint = {
        let show_fingerprint = show_fingerprint.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            show_fingerprint.set(!*show_fingerprint);
        })
    };

    let mailto = format!("mailto:{}", CONFIG.email);
    let url_display = CONFIG
        .url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    html! {
        <section id={ section_id("contact") } class="sec" ref={section}>
            <Reveal>
                <SectionHeader num={ section_label("contact") }
                    title={ i18n.t("contact.title") }
                    intro={ i18n.t("contact.intro") } />
            </Reveal>

            <div class="grid-12">
                <div class="col-label" />
                <div class="col-body">
                    <Reveal delay={120}>
                        <div class="terminal-block">
                            <div class="terminal-head">
                                <span class="term-dots"><span /><span /><span /></span>
                                <span class="mono text-muted">{"~/contact — zsh"}</span>
                                <span />
                            </div>
                            <div class="terminal-body">
                                <div class="term-line">
                                    <span class="term-prompt">{"$"}</span>
                                    <span class="term-cmd">{(*typed).clone()}<span class="term-cursor">{"▊"}</span></span>
                                </div>
                                <div class="term-output">
                                    <span class="mono text-muted">{"connecting..."}</span>
                                    <span class="mono text-accent ml-3">{"ok"}</span>
                                </div>
                            </div>
                        </div>
                    </Reveal>

                    <Reveal delay={200}>
                        <div class="contact-email-display">
                            <span class="email-prefix text-accent">{">_"}</span>
                            <a href={mailto.clone()} class="email-link">{CONFIG.email}</a>
                        </div>
                    </Reveal>

                    <Reveal delay={260}>
                        <div class="contact-actions">
                            <button onclick={copy_email} class="btn-accent">
                                <span class="mono">{ if *copied { i18n.t("contact.copied") } else { i18n.t("contact.copy") } }</span>
                            </button>
                            <a href={mailto} class="btn-outline">
                                <span class="mono">{ format!("{} →", i18n.t("contact.write")) }</span>
                            </a>
                            <a href={CONFIG.github} target="_blank" rel="noreferrer" class="btn-outline">
                                <span class="mono">{"GitHub →"}</span>
                            </a>
                            <a href={CONFIG.linkedin} target="_blank" rel="noreferrer" class="btn-outline">
                                <span class="mono">{"LinkedIn →"}</span>
                            </a>
                            <div class="resume-btn-group">
                                <a href={format!("/resume/{resume_name}")} target="_blank" rel="noopener" class="btn-outline resume-dl">
                                    <span class="mono">{ format!("{resume_label} ↓") }</span>
                                </a>
                                if let Some((algorithm, digest)) = resume_digest.clone() {
                                    <button type="button"
                                            class="resume-info-btn"
                                            onclick={toggle_fingerprint}
                                            aria-label={ format!("{algorithm} fingerprint") }
                                            title={ format!("{algorithm} fingerprint") }>
                                        <span class="mono">{"ⓘ"}</span>
                                    </button>
                                    if *show_fingerprint {
                                        <div class="fp-popup">
                                            <span class="mono text-muted">{ format!("{algorithm} fingerprint") }</span>
                                            <div class="fp-row" title={digest.clone()}>
                                                <span class="text-fg/80 shrink-0">{resume_name}</span>
                                                <span class="fp-digest">{digest}</span>
                                            </div>
                                            <span class="text-muted text-[11.5px]">{ i18n.t("contact.fingerprintNote") }</span>
                                            if let Some(sig) = resume_signature.clone() {
                                                <hr class="fp-sep" />
                                                <span class="mono text-muted">{ i18n.t("contact.signatureTitle") }</span>
                                                <div class="fp-row" title={sig.identity.clone()}>
                                                    <span class="text-fg/80 shrink-0">{ i18n.t("contact.signatureIdentity") }</span>
                                                    <span class="fp-digest">{sig.identity.clone()}</span>
                                                </div>
                                                <div class="fp-row" title={sig.issuer.clone()}>
                                                    <span class="text-fg/80 shrink-0">{ i18n.t("contact.signatureIssuer") }</span>
                                                    <span class="fp-digest">{sig.issuer.clone()}</span>
                                                </div>
                                                if let Some(log_url) = sig.rekor_log_url.clone() {
                                                    <div class="fp-row">
                                                        <span class="text-fg/80 shrink-0">{ i18n.t("contact.signatureLog") }</span>
                                                        <a class="fp-digest" href={log_url.clone()} target="_blank" rel="noreferrer">{log_url}</a>
                                                    </div>
                                                }
                                                <span class="text-muted text-[11.5px]">{ i18n.t("contact.signatureNote") }</span>
                                            }
                                        </div>
                                    }
                                }
                            </div>
                            <a href={CONFIG.url} target="_blank" rel="noreferrer" class="btn-outline">
                                <span class="mono">{ format!("{url_display} ↗") }</span>
                            </a>
                        </div>
                    </Reveal>

                </div>
            </div>
        </section>
    }
}
