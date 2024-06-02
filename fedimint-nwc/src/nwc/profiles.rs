use core::fmt;
use std::cmp::Ordering;
use std::str::FromStr;

use itertools::Itertools;
use lightning_invoice::Bolt11Invoice;
use multimint::fedimint_ln_common::bitcoin::util::bip32::ExtendedPrivKey;
use nostr::nips::nip04::encrypt;
use nostr::nips::nip47::*;
use nostr::{Event, EventBuilder, EventId, Filter, JsonUtil, Keys, Kind, Tag, Timestamp};
use nostr_sdk::secp256k1::{Secp256k1, Signing};
use serde::{Deserialize, Serialize};
use tracing::error;

use super::conditions::SpendingConditions;
use crate::services::nostr::derive_nwc_keys;
use crate::utils;

/// Type of Nostr Wallet Connect profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NwcProfileTag {
    Subscription,
    Gift,
    General,
}

impl Default for NwcProfileTag {
    fn default() -> Self {
        Self::General
    }
}

impl fmt::Display for NwcProfileTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subscription => write!(f, "Subscription"),
            Self::Gift => write!(f, "Gift"),
            Self::General => write!(f, "General"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct Profile {
    pub name: String,
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<nostr::PublicKey>,
    pub relay: String,
    pub enabled: Option<bool>,
    /// Archived profiles will not be displayed
    pub archived: Option<bool>,
    /// Require approval before sending a payment
    #[serde(default)]
    pub spending_conditions: SpendingConditions,
    /// Allowed commands for this profile
    pub(crate) commands: Option<Vec<Method>>,
    /// index to use to derive nostr keys for child index
    /// set to Option so that we keep using `index` for reserved + existing
    #[serde(default)]
    pub child_key_index: Option<u32>,
    #[serde(default)]
    pub tag: NwcProfileTag,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl Profile {
    pub fn active(&self) -> bool {
        match (self.enabled, self.archived) {
            (Some(enabled), Some(archived)) => enabled && !archived,
            (Some(enabled), None) => enabled,
            (None, Some(archived)) => !archived,
            (None, None) => true,
        }
    }

    /// Returns the available commands for this profile
    pub fn available_commands(&self) -> &[Method] {
        // if None this is an old profile and we should only allow pay invoice
        match self.commands.as_ref() {
            None => &[Method::PayInvoice],
            Some(cmds) => cmds,
        }
    }
}

impl PartialOrd for Profile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

#[derive(Clone)]
pub(crate) struct NostrWalletConnect {
    /// Client key used for Nostr Wallet Connect.
    /// Given to the client in the connect URI.
    pub(crate) client_key: Keys,
    /// Server key used for Nostr Wallet Connect.
    /// Used by the nostr client to encrypt messages to the wallet.
    /// Used by the server to decrypt messages from the nostr client.
    pub(crate) server_key: Keys,
    pub(crate) profile: Profile,
}

impl NostrWalletConnect {
    pub fn new<C: Signing>(
        context: &Secp256k1<C>,
        xprivkey: ExtendedPrivKey,
        profile: Profile,
    ) -> Result<NostrWalletConnect, anyhow::Error> {
        let key_derivation_index = profile.child_key_index.unwrap_or(profile.index);

        let (derived_client_key, server_key) =
            derive_nwc_keys(context, xprivkey, key_derivation_index)?;

        // if the profile has a client key, we should use that instead of the derived
        // one, that means that the profile was created from NWA
        let client_key = match profile.client_key {
            Some(client_key) => Keys::from_public_key(client_key),
            None => derived_client_key,
        };

        Ok(Self {
            client_key,
            server_key,
            profile,
        })
    }

    pub fn get_nwc_uri(&self) -> anyhow::Result<Option<NostrWalletConnectURI>> {
        match self.client_key.secret_key().ok() {
            Some(sk) => Ok(Some(NostrWalletConnectURI::new(
                self.server_key.public_key(),
                self.profile.relay.parse()?,
                sk.clone(),
                None,
            ))),
            None => Ok(None),
        }
    }

    pub fn client_pubkey(&self) -> nostr::PublicKey {
        self.client_key.public_key()
    }

    pub fn server_pubkey(&self) -> nostr::PublicKey {
        self.server_key.public_key()
    }

    pub fn create_nwc_filter(&self, timestamp: Timestamp) -> Filter {
        Filter::new()
            .kinds(vec![Kind::WalletConnectRequest])
            .author(self.client_pubkey())
            .pubkey(self.server_pubkey())
            .since(timestamp)
    }

    /// Create Nostr Wallet Connect Info event
    pub fn create_nwc_info_event(&self) -> anyhow::Result<Event> {
        let commands = self
            .profile
            .available_commands()
            .iter()
            .map(|c| c.to_string())
            .join(" ");
        let info =
            EventBuilder::new(Kind::WalletConnectInfo, commands, []).to_event(&self.server_key)?;
        Ok(info)
    }

    /// Create Nostr Wallet Auth Confirmation event
    pub fn create_auth_confirmation_event(
        &self,
        uri_relay: Url,
        secret: String,
        commands: Vec<Method>,
    ) -> anyhow::Result<Option<Event>> {
        // skip non-NWA profiles
        if self.profile.client_key.is_none() {
            return Ok(None);
        }

        // if the relay is the same as the profile, we don't need to send it
        let relay = if uri_relay == Url::parse(&self.profile.relay)? {
            None
        } else {
            Some(self.profile.relay.clone())
        };

        let json = NIP49Confirmation {
            secret,
            commands,
            relay,
        };
        let content = encrypt(
            self.server_key.secret_key()?,
            &self.client_pubkey(),
            serde_json::to_string(&json)?,
        )?;
        let d_tag = Tag::Identifier(self.client_pubkey().to_hex());
        let event = EventBuilder::new(Kind::ParameterizedReplaceable(33194), content, [d_tag])
            .to_event(&self.server_key)?;
        Ok(Some(event))
    }

    pub(crate) async fn pay_nwc_invoice(
        &self,
        node: &impl InvoiceHandler,
        invoice: &Bolt11Invoice,
    ) -> Result<Response, anyhow::Error> {
        let label = self
            .profile
            .label
            .clone()
            .unwrap_or(self.profile.name.clone());
        match node.pay_invoice(invoice, None, vec![label]).await {
            Ok(inv) => {
                // preimage should be set after a successful payment
                let preimage = inv.preimage.ok_or(anyhow::anyhow!("preimage not set"))?;
                Ok(Response {
                    result_type: Method::PayInvoice,
                    error: None,
                    result: Some(ResponseResult::PayInvoice(PayInvoiceResponseResult {
                        preimage,
                    })),
                })
            }
            Err(e) => {
                error!("failed to pay invoice: {e}");
                Err(e)
            }
        }
    }

    async fn save_pending_nwc_invoice<S: MutinyStorage, P: PrimalApi, C: NostrClient>(
        &self,
        nostr_manager: &NostrManager<S, P, C>,
        event_id: EventId,
        event_pk: nostr::PublicKey,
        invoice: Bolt11Invoice,
        identifier: Option<String>,
    ) -> anyhow::Result<()> {
        nostr_manager
            .save_pending_nwc_invoice(
                Some(self.profile.index),
                event_id,
                event_pk,
                invoice,
                identifier,
            )
            .await
    }

    fn get_skipped_error_event(
        &self,
        event: &Event,
        result_type: Method,
        error_code: ErrorCode,
        message: String,
    ) -> anyhow::Result<Event> {
        let server_key = self.server_key.secret_key()?;
        let client_pubkey = self.client_key.public_key();
        let content = Response {
            result_type,
            error: Some(NIP47Error {
                code: error_code,
                message,
            }),
            result: None,
        };

        let encrypted = encrypt(server_key, &client_pubkey, content.as_json())?;

        let p_tag = Tag::public_key(event.pubkey);
        let e_tag = Tag::event(event.id);
        let response = EventBuilder::new(Kind::WalletConnectResponse, encrypted, [p_tag, e_tag])
            .to_event(&self.server_key)?;

        Ok(response)
    }

    pub fn nwc_profile(&self) -> NwcProfile {
        NwcProfile {
            name: self.profile.name.clone(),
            index: self.profile.index,
            client_key: self.profile.client_key,
            relay: self.profile.relay.clone(),
            enabled: self.profile.enabled,
            archived: self.profile.archived,
            nwc_uri: match self.get_nwc_uri() {
                Ok(Some(uri)) => Some(uri.to_string()),
                _ => {
                    error!("Failed to get nwc uri");
                    None
                }
            },
            spending_conditions: self.profile.spending_conditions.clone(),
            commands: self.profile.commands.clone(),
            child_key_index: self.profile.child_key_index,
            tag: self.profile.tag,
            label: self.profile.label.clone(),
        }
    }
}

/// Struct for externally exposing a nostr wallet connect profile
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NwcProfile {
    pub name: String,
    pub index: u32,
    /// Public Key given in a Nostr Wallet Auth URI.
    /// This will only be defined for profiles created through Nostr Wallet
    /// Auth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<nostr::PublicKey>,
    pub relay: String,
    pub enabled: Option<bool>,
    pub archived: Option<bool>,
    /// Nostr Wallet Connect URI
    /// This will only be defined for profiles created manually.
    pub nwc_uri: Option<String>,
    #[serde(default)]
    pub spending_conditions: SpendingConditions,
    /// Allowed commands for this profile
    pub commands: Option<Vec<Method>>,
    #[serde(default)]
    pub child_key_index: Option<u32>,
    #[serde(default)]
    pub tag: NwcProfileTag,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl NwcProfile {
    pub(crate) fn profile(&self) -> Profile {
        Profile {
            name: self.name.clone(),
            index: self.index,
            client_key: self.client_key,
            relay: self.relay.clone(),
            archived: self.archived,
            enabled: self.enabled,
            spending_conditions: self.spending_conditions.clone(),
            commands: self.commands.clone(),
            child_key_index: self.child_key_index,
            tag: self.tag,
            label: self.label.clone(),
        }
    }
}

/// An invoice received over Nostr Wallet Connect that is pending approval or
/// rejection
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PendingNwcInvoice {
    /// Index of the profile that received the invoice.
    /// None if invoice is from a DM
    pub index: Option<u32>,
    /// The invoice that awaiting approval
    pub invoice: Bolt11Invoice,
    /// The nostr event id of the request
    pub event_id: EventId,
    /// The nostr pubkey of the request
    /// If this is a DM, this is who sent us the request
    pub pubkey: nostr::PublicKey,
    /// `id` parameter given in the original request
    /// This is normally only given for MultiPayInvoice requests
    pub identifier: Option<String>,
}

impl PartialOrd for PendingNwcInvoice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PendingNwcInvoice {
    fn cmp(&self, other: &Self) -> Ordering {
        self.invoice.to_string().cmp(&other.invoice.to_string())
    }
}

impl PendingNwcInvoice {
    pub fn is_expired(&self) -> bool {
        self.invoice.would_expire(utils::now())
    }
}

/// Checks if it is a valid invoice
/// Return an error string if invalid
/// Otherwise returns an optional invoice that should be processed
pub(crate) async fn check_valid_nwc_invoice(
    params: &PayInvoiceRequestParams,
    invoice_handler: &impl InvoiceHandler,
) -> Result<Option<Bolt11Invoice>, String> {
    let invoice = match Bolt11Invoice::from_str(&params.invoice) {
        Ok(invoice) => invoice,
        Err(_) => return Err("Invalid invoice".to_string()),
    };

    // if the invoice has expired, skip it
    if invoice.would_expire(utils::now()) {
        return Err("Invoice expired".to_string());
    }

    // if the invoice has no amount, we cannot pay it
    if invoice.amount_milli_satoshis().is_none() {
        log_warn!(
            invoice_handler.logger(),
            "NWC Invoice amount not set, cannot pay: {invoice}"
        );

        if params.amount.is_none() {
            return Err("Invoice amount not set".to_string());
        }

        // TODO we cannot pay invoices with msat values so for now return an error
        return Err("Paying 0 amount invoices is not supported yet".to_string());
    }

    if invoice_handler.skip_hodl_invoices() {
        // Skip potential hodl invoices as they can cause force closes
        if utils::is_hodl_invoice(&invoice) {
            log_warn!(
                invoice_handler.logger(),
                "Received potential hodl invoice, skipping..."
            );
            return Err("Paying hodl invoices disabled".to_string());
        }
    }

    // if we have already paid or are attempting to pay this invoice, skip it
    if invoice_handler
        .lookup_payment(&invoice.payment_hash().into_32())
        .await
        .map(|i| i.status)
        .is_some_and(|status| matches!(status, HTLCStatus::Succeeded | HTLCStatus::InFlight))
    {
        return Ok(None);
    }

    Ok(Some(invoice))
}
