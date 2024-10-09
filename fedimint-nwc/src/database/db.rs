use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use itertools::Itertools;
use multimint::fedimint_ln_common::lightning_invoice::Bolt11Invoice;
use nostr::nips::nip47::LookupInvoiceRequestParams;
use nostr::util::hex;
use redb::{Database as RedbDatabase, ReadTransaction, ReadableTable, WriteTransaction};

use super::invoice::{Invoice, INVOICES_TABLE};
use super::payment::{Payment, PAYMENTS_TABLE};

/// Database for storing and retrieving payment information
/// Invoices are invoices that we create as part of make_invoice
/// Payments are payments that we perform as part of pay_invoice
/// Any other configs here are just temporary until we have a better way to
/// store them for making the more complex rate limiting and payments caveats
/// for more interesting NWC usecases
#[derive(Debug, Clone)]
pub struct Database {
    db: Arc<RedbDatabase>,
    pub max_amount: u64,
    pub daily_limit: u64,
    _rate_limit: Duration,
}

impl From<RedbDatabase> for Database {
    fn from(db: RedbDatabase) -> Self {
        Self {
            db: Arc::new(db),
            max_amount: 0,
            daily_limit: 0,
            _rate_limit: Duration::from_secs(0),
        }
    }
}

impl Database {
    pub fn new(
        redb_path: &PathBuf,
        max_amount: u64,
        daily_limit: u64,
        rate_limit_secs: u64,
    ) -> Result<Self> {
        let db = RedbDatabase::create(redb_path)
            .with_context(|| format!("Failed to create redb at {}", redb_path.display()))?;
        Ok(Self {
            db: Arc::new(db),
            max_amount,
            daily_limit,
            _rate_limit: Duration::from_secs(rate_limit_secs),
        })
    }

    pub fn write_with<R>(&self, f: impl FnOnce(&'_ WriteTransaction) -> Result<R>) -> Result<R> {
        let mut dbtx = self.db.begin_write()?;
        let res = f(&mut dbtx)?;
        dbtx.commit()?;
        Ok(res)
    }

    pub fn read_with<R>(&self, f: impl FnOnce(&'_ ReadTransaction) -> Result<R>) -> Result<R> {
        let dbtx = self.db.begin_read()?;
        f(&dbtx)
    }

    pub fn add_payment(&self, invoice: Bolt11Invoice) -> Result<()> {
        let payment_hash_encoded = hex::encode(invoice.payment_hash());
        self.write_with(|dbtx| {
            let mut payments = dbtx.open_table(PAYMENTS_TABLE)?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            let payment = Payment::new(now, invoice.amount_milli_satoshis().unwrap_or(0), invoice);
            payments.insert(&payment_hash_encoded.as_str(), &payment)?;
            Ok(())
        })
    }

    pub fn sum_payments(&self) -> Result<u64> {
        self.read_with(|dbtx| {
            let payments = dbtx.open_table(PAYMENTS_TABLE)?;
            payments
                .iter()?
                .map_ok(|(_, v)| v.value().amount)
                .try_fold(0u64, |acc, amount| amount.map(|amt| acc + amt))
                .map_err(anyhow::Error::new)
        })
    }

    pub fn check_payment_limits(&self, msats: u64) -> Result<(), anyhow::Error> {
        let total_msats = self.sum_payments().unwrap_or(0) * 1_000;
        if self.max_amount > 0 && msats > self.max_amount * 1_000 {
            Err(anyhow::Error::msg("Invoice amount too high."))
        } else if self.daily_limit > 0 && total_msats + msats > self.daily_limit * 1_000 {
            Err(anyhow::Error::msg("Daily limit exceeded."))
        } else {
            Ok(())
        }
    }

    pub fn add_invoice(&self, invoice: &Bolt11Invoice) -> Result<()> {
        let payment_hash_encoded = hex::encode(invoice.payment_hash());
        let invoice = Invoice::from(invoice);
        self.write_with(|dbtx| {
            let mut invoices = dbtx.open_table(INVOICES_TABLE)?;
            invoices
                .insert(&payment_hash_encoded.as_str(), &invoice)
                .map_err(anyhow::Error::new)
                .map(|_| ())
        })
    }

    pub fn lookup_invoice(&self, params: LookupInvoiceRequestParams) -> Result<Invoice> {
        if let Some(payment_hash) = params.payment_hash {
            let payment_hash_encoded = hex::encode(payment_hash);
            self.read_with(|dbtx| {
                let invoices = dbtx.open_table(INVOICES_TABLE)?;
                invoices
                    .get(&payment_hash_encoded.as_str())
                    .map_err(anyhow::Error::new)
                    .and_then(|opt_invoice| {
                        opt_invoice
                            .ok_or_else(|| anyhow::Error::msg("Invoice not found"))
                            .map(|access_guard| access_guard.value().clone())
                    })
            })
        } else if let Some(bolt11) = params.invoice {
            let invoice = Bolt11Invoice::from_str(&bolt11).map_err(anyhow::Error::new)?;
            let payment_hash_encoded = hex::encode(invoice.payment_hash());
            self.read_with(|dbtx| {
                let invoices = dbtx.open_table(INVOICES_TABLE)?;
                invoices
                    .get(&payment_hash_encoded.as_str())
                    .map_err(anyhow::Error::new)
                    .and_then(|opt_invoice| {
                        opt_invoice
                            .ok_or_else(|| anyhow::Error::msg("Invoice not found"))
                            .map(|access_guard| access_guard.value().clone())
                    })
            })
        } else {
            Err(anyhow::Error::msg("No invoice or payment hash provided"))
        }
    }
}
