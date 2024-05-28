
/** --- TOAST --- */

function escapeHtml(html) {
  const div = document.createElement('div');
  div.textContent = html;
  return div.innerHTML;
}

/**
 * Shows a toast
 * @param {string} message
 * @param {"primary" | "success" | "neutral" | "warning" | "danger" | undefined} variant
 * @param {string?} icon
 * @param {number?} duration
 */
const dispatchToast = (message, variant = "primary", icon = 'info-circle', duration = 3000) => {
  const container = document.querySelector('.alert-toast');
  const alert = Object.assign(document.createElement('sl-alert'), {
    variant,
    closable: true,
    duration: duration,
    innerHTML: `
      <sl-icon name="${icon}" slot="icon"></sl-icon>
      ${escapeHtml(message)}
    `
  });

  document.body.append(alert)
  return alert.toast()
}

/** --- FORM --- */

const setupForm = () => {

  const form = document.querySelector('.input-form');
  const data = new FormData(form);

  Promise.all([
    customElements.whenDefined('sl-button'),
    customElements.whenDefined('sl-checkbox'),
    customElements.whenDefined('sl-input'),
    customElements.whenDefined('sl-option'),
    customElements.whenDefined('sl-select'),
    customElements.whenDefined('sl-textarea')
  ]).then(() => {
    form.addEventListener('submit', event => {
      event.preventDefault();
      alert('All fields are valid!');
    })
  })
}
setupForm()

/** --- QR --- */
const loadQr = () => {
  const qrCode = document.querySelector('.qr-code');
}

/** --- Dialog --- */

const setupWalletDialog = () => {
  const wallet = document.querySelector('.wallet-card')
  const createInvoiceButton = wallet.querySelector('.create-invoice')
  const payInvoiceButton = wallet.querySelector('.pay-invoice')

  const createDialog = document.querySelector('.create-invoice-dialog');
  const payDialog = document.querySelector('.pay-invoice-dialog');
  const generateInvoiceButton = createDialog.querySelector('sl-button[slot="footer"]');

  createInvoiceButton.addEventListener('click', () => createDialog.show());
  payInvoiceButton.addEventListener('click', () => dispatchToast('Unimplemented'));
  // payInvoiceButton.addEventListener('click', () => payDialog.show());

  const form = document.querySelector('.invoice-form');
  const data = new FormData(form);
  Promise.all([
    customElements.whenDefined('sl-button'),
    customElements.whenDefined('sl-checkbox'),
    customElements.whenDefined('sl-input'),
    customElements.whenDefined('sl-option'),
    customElements.whenDefined('sl-select'),
    customElements.whenDefined('sl-textarea')
  ]).then(() => {
    form.addEventListener('submit', event => {
      event.preventDefault();
      alert('All fields are valid!');
    })
    generateInvoiceButton.addEventListener('click', () => dispatchToast("Invoice Created! (fake)", 'success', 'check2-circle'));
  })
}
setupWalletDialog()
