'use client'
import { useEffect } from 'react'

export default function HideConsentBrandingFooter() {
  useEffect(() => {
    // Strictly target the footer inside the consent dialog
    const footerSelector = '[data-testid="consent-manager-dialog-footer"]'

    const removeFooter = () => {
      const footer = document.querySelector(footerSelector)
      if (footer) footer.remove()
    }

    // Remove immediately if already in DOM
    removeFooter()

    // Observe the DOM for dynamically inserted footer
    const observer = new MutationObserver(() => {
      removeFooter()
    })

    observer.observe(document.body, { childList: true, subtree: true })

    // Inject CSS scoped to the footer only
    const style = document.createElement('style')
    style.id = 'hide-c15t-footer'
    style.textContent = `
      ${footerSelector} {
        display: none !important;
        visibility: hidden !important;
      }
    `
    document.head.appendChild(style)

    return () => {
      observer.disconnect()
      document.head.removeChild(style)
    }
  }, [])

  return null
}
