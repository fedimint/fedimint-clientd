use std::time::{Duration, UNIX_EPOCH};

use lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescription};
use nostr::util::hex;
use redb::{TableDefinition, TypeName, Value};
use serde::{Deserialize, Serialize};

pub const INVOICES_TABLE: TableDefinition<&str, Invoice> = TableDefinition::new("invoices");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice: Bolt11Invoice,
    pub preimage: Option<String>,
    pub settle_date: Option<u64>,
}

impl Invoice {
    pub fn created_at(&self) -> u64 {
        self.invoice
            .timestamp()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn expires_at(&self) -> u64 {
        self.created_at() + self.invoice.expiry_time().as_secs()
    }

    pub fn settled_at(&self) -> Option<u64> {
        self.settle_date
            .map(|time| UNIX_EPOCH + Duration::from_secs(time))
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
    }

    pub fn payment_hash(&self) -> String {
        hex::encode(self.invoice.payment_hash().to_vec())
    }

    pub fn description(&self) -> Option<Bolt11InvoiceDescription> {
        Some(self.invoice.description())
    }
}

impl From<&Bolt11Invoice> for Invoice {
    fn from(invoice: &Bolt11Invoice) -> Self {
        Self {
            invoice: invoice.clone(),
            preimage: None,
            settle_date: None,
        }
    }
}

impl Value for Invoice {
    type SelfType<'a> = Self where Self: 'a;
    type AsBytes<'a> = Vec<u8> where Self: 'a;

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Vec<u8> {
        // nosemgrep: use-of-unwrap
        bincode::serialize(value).unwrap()
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        // nosemgrep: use-of-unwrap
        bincode::deserialize(data).unwrap()
    }

    fn fixed_width() -> Option<usize> {
        None // Return Some(width) if fixed width, None if variable width
    }

    fn type_name() -> TypeName {
        TypeName::new("Invoice")
    }
}
