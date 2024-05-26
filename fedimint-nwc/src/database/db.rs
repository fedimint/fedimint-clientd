use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use itertools::Itertools;
use redb::{Database as RedbDatabase, ReadTransaction, ReadableTable, WriteTransaction};

use super::payment::{Payment, PAYMENTS_TABLE};

#[derive(Debug, Clone)]
pub struct Database {
    db: Arc<RedbDatabase>,
    max_amount: u64,
    daily_limit: u64,
    rate_limit: Duration,
}

impl From<RedbDatabase> for Database {
    fn from(db: RedbDatabase) -> Self {
        Self {
            db: Arc::new(db),
            max_amount: 0,
            daily_limit: 0,
            rate_limit: Duration::from_secs(0),
        }
    }
}

impl Database {
    pub fn new(
        db_path: &PathBuf,
        max_amount: u64,
        daily_limit: u64,
        rate_limit_secs: u64,
    ) -> Result<Self> {
        let db = RedbDatabase::create(db_path)
            .with_context(|| format!("Failed to create database at {}", db_path.display()))?;
        Ok(Self {
            db: Arc::new(db),
            max_amount,
            daily_limit,
            rate_limit: Duration::from_secs(rate_limit_secs),
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

    pub fn add_payment(&self, amount: u64, destination: String) -> Result<()> {
        self.write_with(|dbtx| {
            let mut payments = dbtx.open_table(PAYMENTS_TABLE)?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            let payment = Payment::new(now, amount, &destination);
            payments.insert(&now, &payment)?;
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

    pub fn check_payment_limits(&self, msats: u64, _dest: String) -> Result<Option<String>> {
        let total_msats = self.sum_payments()? * 1_000;
        if self.max_amount > 0 && msats > self.max_amount * 1_000 {
            Ok(Some("Invoice amount too high.".to_string()))
        } else if self.daily_limit > 0 && total_msats + msats > self.daily_limit * 1_000 {
            Ok(Some("Daily limit exceeded.".to_string()))
        } else {
            Ok(None)
        }
    }
}
