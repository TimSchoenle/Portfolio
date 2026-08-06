use dioxus::prelude::*;
use portfolio_data::CONFIG;

use super::legal::{AddressBlock, LegalPage, LegalSection};
use crate::i18n::use_i18n;

/// Sections rendered as plain heading/body pairs, in order.
const SECTIONS: [(&str, &str); 9] = [
    ("privacy.generalHeading", "privacy.generalBody"),
    ("privacy.hostingHeading", "privacy.hostingBody"),
    ("privacy.encryptionHeading", "privacy.encryptionBody"),
    ("privacy.logsHeading", "privacy.logsBody"),
    ("privacy.cloudflareHeading", "privacy.cloudflareBody"),
    ("privacy.contactHeading", "privacy.contactBody"),
    ("privacy.rightsHeading", "privacy.rightsBody"),
    ("privacy.storageHeading", "privacy.storageBody"),
    ("privacy.noTrackingHeading", "privacy.noTrackingBody"),
];

#[component]
pub fn Privacy() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    rsx! {
        LegalPage {
            title: t("privacy.title"),
            last_updated: CONFIG.legal.privacy_last_change,
            canonical_path: "/privacy",
            p { class: "text-fg/85 leading-relaxed", {t("privacy.intro")} }

            LegalSection { heading: t("privacy.controllerHeading"), body: Some(t("privacy.controllerBody")),
                AddressBlock {}
            }

            {SECTIONS.iter().map(|(heading, body)| rsx! {
                LegalSection { key: "{heading}", heading: t(heading), body: Some(t(body)) }
            })}

            LegalSection { heading: t("privacy.changesHeading"), body: Some(t("privacy.changesBody")) }
        }
    }
}
