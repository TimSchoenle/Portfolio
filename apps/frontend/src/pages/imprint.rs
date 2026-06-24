use i18nrs::yew::use_translation;
use portfolio_data::CONFIG;
use yew::prelude::*;

use super::legal::{AddressBlock, LegalPage, LegalSection};

#[function_component(Imprint)]
pub fn imprint() -> Html {
    let (i18n, _) = use_translation();

    html! {
        <LegalPage title={ i18n.t("imprint.title") } last_updated={CONFIG.legal.imprint_last_change}>
            <LegalSection heading={ i18n.t("imprint.tmgHeading") }>
                <AddressBlock />
            </LegalSection>

            <LegalSection heading={ i18n.t("imprint.vatHeading") }>
                <p>{ i18n.t("imprint.vatBody") }</p>
                <p class="font-mono text-fg">{ CONFIG.legal.vat_id }</p>
            </LegalSection>

            <LegalSection heading={ i18n.t("imprint.mstvHeading") }>
                <AddressBlock />
            </LegalSection>

            <LegalSection heading={ i18n.t("imprint.disputeHeading") } body={ i18n.t("imprint.disputeBody") } />

            <LegalSection heading={ i18n.t("imprint.socialHeading") }>
                <ul class="list-disc pl-5 space-y-1.5">
                    <li>
                        <a class="text-accent hover:underline" href={CONFIG.github} target="_blank" rel="noopener">
                            {CONFIG.github}
                        </a>
                    </li>
                    <li>
                        <a class="text-accent hover:underline" href={CONFIG.linkedin} target="_blank" rel="noopener">
                            {CONFIG.linkedin}
                        </a>
                    </li>
                </ul>
            </LegalSection>

            <LegalSection heading={ i18n.t("imprint.liabilityContentHeading") } body={ i18n.t("imprint.liabilityContentBody") } />
            <LegalSection heading={ i18n.t("imprint.liabilityLinksHeading") } body={ i18n.t("imprint.liabilityLinksBody") } />
            <LegalSection heading={ i18n.t("imprint.copyrightHeading") } body={ i18n.t("imprint.copyrightBody") } />
        </LegalPage>
    }
}
