use i18nrs::yew::use_translation;
use portfolio_data::CONFIG;
use yew::prelude::*;

use super::legal::{AddressBlock, LegalPage, LegalSection};

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

#[function_component(PrivacyPolicy)]
pub fn privacy_policy() -> Html {
    let (i18n, _) = use_translation();

    html! {
        <LegalPage title={ i18n.t("privacy.title") } last_updated={CONFIG.legal.privacy_last_change}>
            <p class="text-fg/85 leading-relaxed">{ i18n.t("privacy.intro") }</p>

            <LegalSection heading={ i18n.t("privacy.controllerHeading") } body={ i18n.t("privacy.controllerBody") }>
                <AddressBlock />
            </LegalSection>

            { for SECTIONS.iter().map(|(heading, body)| html!{
                <LegalSection heading={ i18n.t(heading) } body={ i18n.t(body) } />
            })}

            <LegalSection heading={ i18n.t("privacy.changesHeading") } body={ i18n.t("privacy.changesBody") } />
        </LegalPage>
    }
}
