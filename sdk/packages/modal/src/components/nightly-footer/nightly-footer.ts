import { customElement, property } from 'lit/decorators.js'
import { tailwindElement } from '../../shared/tailwind.element'
import style from './nightly-footer.css'
import { LitElement, html } from 'lit'
import { type FooterData, type FooterLink } from '../../utils/types'

@customElement('nightly-footer')
export class NightlyFooter extends LitElement {
  static styles = tailwindElement(style)

  @property({ type: Object })
  footerDataOverride: FooterData | undefined = undefined

  render() {
    return html`
      <div class="nc_footer">
        ${this.footerDataOverride?.overrideContent
          ? this.footerDataOverride.overrideContent.map(
              (link: FooterLink) => html`
                ${' ' + link.description.trim() + ' '}
                <a href="${link.linkUrl}" class="nc_footerLink">${link.linkName}</a>
              `
            )
          : html` By connecting, you agree to Common's
              <a
                href="${
                  this.footerDataOverride?.defaultContentTermsLink
                    ? this.footerDataOverride?.defaultContentTermsLink
                    : '/defaultTerms'
                }"
                class="nc_footerLink"
                >Terms of Service</a
              >
              and to its <a href="${
                this.footerDataOverride?.defaultContentPrivacyLink
                  ? this.footerDataOverride?.defaultContentPrivacyLink
                  : '/defaultPrivacy'
              }"" class="nc_footerLink">Privacy Policy</a>.`}
      </div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'nightly-footer': NightlyFooter
  }
}
