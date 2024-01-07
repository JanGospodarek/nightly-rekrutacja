import { customElement, property } from 'lit/decorators.js'
import { tailwindElement } from '../../shared/tailwind.element'
import style from './nightly-footer.css'
import { LitElement, html } from 'lit'
import { type FooterData, type FooterLink } from '@nightlylabs/wallet-selector-base'

@customElement('nightly-footer')
export class NightlyFooter extends LitElement {
  static styles = tailwindElement(style)

  //   @property()
  //   // eslint-disable-next-line @typescript-eslint/no-empty-function
  //   onClose = () => {}

  @property({ type: Object })
  optionalFooterData: FooterData | undefined = undefined

  render() {
    return html`
      <div class="nc_footer">
        ${this.optionalFooterData
          ? html`
              ${this.optionalFooterData.map(
                (link: FooterLink) => html`
                  ${link.describtion}
                  <span class="nc_footerLink">${link.hrefName}</span>
                `
              )}
            `
          : html` By connecting, you agree to Common's
              <span class="nc_footerLink">Terms of Service</span> and to its
              <span class="nc_footerLink">Privacy Policy</span>.`}
      </div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'nightly-footer': NightlyFooter
  }
}
