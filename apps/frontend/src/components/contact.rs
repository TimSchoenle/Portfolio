//! Contact section from the v4 design: terminal with a type-in `ssh` command,
//! oversized email link, action buttons — plus the localized resume PDFs with
//! their SHA-256 fingerprints.

use std::cell::RefCell;
use std::rc::Rc;

use i18nrs::yew::use_translation;
use portfolio_data::{CONFIG, resume_file};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::github::{ResumeFingerprints, load_resume_fingerprints};

use super::reveal::Reveal;
use super::section_header::SectionHeader;

#[function_component(Contact)]
pub fn contact() -> Html {
    let (i18n, _) = use_translation();
    let copied = use_state(|| false);
    let typed = use_state(String::new);
    let fingerprints: UseStateHandle<Option<ResumeFingerprints>> = use_state(|| None);
    let section = use_node_ref();

    {
        let fingerprints = fingerprints.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                // Absent in dev builds without generated resumes — fine.
                if let Ok(f) = load_resume_fingerprints().await {
                    fingerprints.set(Some(f));
                }
            });
        });
    }

    // Type the `ssh` command once the section scrolls into view.
    {
        let typed = typed.clone();
        let section = section.clone();
        use_effect_with((), move |_| {
            let observer: Rc<RefCell<Option<web_sys::IntersectionObserver>>> =
                Rc::new(RefCell::new(None));
            let observer_in_cb = observer.clone();
            let cb =
                Closure::<dyn Fn(js_sys::Array)>::wrap(Box::new(move |entries: js_sys::Array| {
                    let entry: web_sys::IntersectionObserverEntry = entries.get(0).unchecked_into();
                    if entry.is_intersecting() {
                        if let Some(obs) = observer_in_cb.borrow_mut().take() {
                            obs.disconnect();
                        }
                        let typed = typed.clone();
                        spawn_local(async move {
                            let target: Vec<char> =
                                format!("ssh {}", CONFIG.email).chars().collect();
                            for i in 0..=target.len() {
                                typed.set(target[..i].iter().collect());
                                let jitter = (js_sys::Math::random() * 40.0) as u32;
                                gloo_timers::future::TimeoutFuture::new(40 + jitter).await;
                            }
                        });
                    }
                }));
            let init = web_sys::IntersectionObserverInit::new();
            init.set_threshold(&JsValue::from_f64(0.3));
            if let (Some(el), Ok(obs)) = (
                section.cast::<web_sys::Element>(),
                web_sys::IntersectionObserver::new_with_options(cb.as_ref().unchecked_ref(), &init),
            ) {
                obs.observe(&el);
                *observer.borrow_mut() = Some(obs);
            }
            move || {
                if let Some(obs) = observer.borrow_mut().take() {
                    obs.disconnect();
                }
                drop(cb);
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

    let mailto = format!("mailto:{}", CONFIG.email);
    let url_display = CONFIG
        .url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    html! {
        <section id="s5" class="sec" ref={section}>
            <Reveal>
                <SectionHeader num="§ 05 — contact"
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
                            <a href={format!("/resume/{}", resume_file("en"))} target="_blank" rel="noopener" class="btn-outline">
                                <span class="mono">{ format!("{} ↓", i18n.t("contact.resumeEn")) }</span>
                            </a>
                            <a href={format!("/resume/{}", resume_file("de"))} target="_blank" rel="noopener" class="btn-outline">
                                <span class="mono">{ format!("{} ↓", i18n.t("contact.resumeDe")) }</span>
                            </a>
                            <a href={CONFIG.url} target="_blank" rel="noreferrer" class="btn-outline">
                                <span class="mono">{ format!("{url_display} ↗") }</span>
                            </a>
                        </div>
                    </Reveal>

                    if let Some(f) = fingerprints.as_ref() {
                        <Reveal delay={320}>
                            <div class="resume-fingerprints">
                                <span class="mono text-muted">{ format!("{} fingerprints", f.algorithm) }</span>
                                { for f.files.iter().map(|(name, digest)| html!{
                                    <div class="fp-row" title={digest.clone()}>
                                        <span class="text-fg/80 shrink-0">{name.clone()}</span>
                                        <span class="fp-digest">{digest.clone()}</span>
                                    </div>
                                })}
                                <span class="text-muted text-[11.5px]">{ i18n.t("contact.fingerprintNote") }</span>
                            </div>
                        </Reveal>
                    }
                </div>
            </div>
        </section>
    }
}
