import { customElement } from 'lit/decorators.js'
import { tailwindElement } from '../../shared/tailwind.element'
import style from './nightly-footer.css'
import { LitElement, html } from 'lit'

@customElement('nightly-footer')
export class NightlyFooter extends LitElement {
  static styles = tailwindElement(style)

  //   @property()
  //   // eslint-disable-next-line @typescript-eslint/no-empty-function
  //   onClose = () => {}

  render() {
    return html`
      <div class="nc_footer">
        By connecting, you agree to Common's <span class="nc_footerLink">Terms of Service</span> and
        to its <span class="nc_footerLink">Privacy Policy</span>.
      </div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'nightly-footer': NightlyFooter
  }
}
