use dioxus::prelude::*;
use portfolio_data::CONFIG;

use super::legal::{AddressBlock, LegalPage, LegalSection};
use crate::i18n::use_i18n;

#[component]
pub fn Imprint() -> Element {
    let i18n = use_i18n().i18n;
    let t = move |k: &str| i18n.read().t(k);

    rsx! {
        LegalPage { title: t("imprint.title"), last_updated: CONFIG.legal.imprint_last_change,
            LegalSection { heading: t("imprint.tmgHeading"),
                AddressBlock {}
            }
            LegalSection { heading: t("imprint.vatHeading"),
                p { {t("imprint.vatBody")} }
                p { class: "font-mono text-fg", "{CONFIG.legal.vat_id}" }
            }
            LegalSection { heading: t("imprint.mstvHeading"),
                AddressBlock {}
            }
            LegalSection { heading: t("imprint.disputeHeading"), body: Some(t("imprint.disputeBody")) }
            LegalSection { heading: t("imprint.socialHeading"),
                ul { class: "list-disc pl-5 space-y-1.5",
                    li {
                        a {
                            class: "text-accent hover:underline",
                            href: CONFIG.github,
                            target: "_blank",
                            rel: "noopener",
                            "{CONFIG.github}"
                        }
                    }
                    li {
                        a {
                            class: "text-accent hover:underline",
                            href: CONFIG.linkedin,
                            target: "_blank",
                            rel: "noopener",
                            "{CONFIG.linkedin}"
                        }
                    }
                }
            }
            LegalSection { heading: t("imprint.liabilityContentHeading"), body: Some(t("imprint.liabilityContentBody")) }
            LegalSection { heading: t("imprint.liabilityLinksHeading"), body: Some(t("imprint.liabilityLinksBody")) }
            LegalSection { heading: t("imprint.copyrightHeading"), body: Some(t("imprint.copyrightBody")) }
        }
    }
}
