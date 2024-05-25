use nostr::nips::nip47::Method;
use nostr_sdk::Event;

use crate::state::AppState;

pub const METHODS: [Method; 8] = [
    Method::GetInfo,
    Method::MakeInvoice,
    Method::GetBalance,
    Method::LookupInvoice,
    Method::PayInvoice,
    Method::MultiPayInvoice,
    Method::PayKeysend,
    Method::MultiPayKeysend,
];

pub async fn handle_nwc_request(_state: &AppState, _event: Event) -> Result<(), anyhow::Error> {
    Ok(())
}
