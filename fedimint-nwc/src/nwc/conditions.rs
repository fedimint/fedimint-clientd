use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use lightning_invoice::Bolt11Invoice;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SingleUseSpendingConditions {
    pub payment_hash: Option<String>,
    pub amount_sats: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackedPayment {
    /// Time in seconds since epoch
    pub time: u64,
    /// Amount in sats
    pub amt: u64,
    /// Payment hash
    pub hash: String,
}

/// When payments for a given payment expire
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BudgetPeriod {
    /// Resets daily at midnight UTC
    Day,
    /// Resets every week on sunday, midnight UTC
    Week,
    /// Resets every month on the first, midnight UTC
    Month,
    /// Resets every year on the January 1st, midnight UTC
    Year,
    /// Payments not older than the given number of seconds are counted
    Seconds(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BudgetedSpendingConditions {
    /// Amount in sats for the allotted budget period
    pub budget: u64,
    /// Max amount in sats for a single payment
    pub single_max: Option<u64>,
    /// Payment history
    pub payments: Vec<TrackedPayment>,
    /// Time period the budget is for
    pub period: BudgetPeriod,
}

impl BudgetedSpendingConditions {
    pub fn add_payment(&mut self, invoice: &Bolt11Invoice) {
        let time = crate::utils::now().as_secs();
        let payment = TrackedPayment {
            time,
            amt: invoice.amount_milli_satoshis().unwrap_or_default() / 1_000,
            hash: invoice.payment_hash().into_32().to_lower_hex_string(),
        };

        self.payments.push(payment);
    }

    pub fn remove_payment(&mut self, invoice: &Bolt11Invoice) {
        let hex = invoice.payment_hash().into_32().to_lower_hex_string();
        self.payments.retain(|p| p.hash != hex);
    }

    fn clean_old_payments(&mut self, now: DateTime<Utc>) {
        let period_start = match self.period {
            BudgetPeriod::Day => now.date_naive().and_hms_opt(0, 0, 0).unwrap_or_default(),
            BudgetPeriod::Week => (now
                - Duration::days((now.weekday().num_days_from_sunday()) as i64))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default(),
            BudgetPeriod::Month => now
                .date_naive()
                .with_day(1)
                .unwrap_or_default()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default(),
            BudgetPeriod::Year => NaiveDateTime::new(
                now.date_naive().with_ordinal(1).unwrap_or_default(),
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default(),
            ),
            BudgetPeriod::Seconds(secs) => now
                .checked_sub_signed(Duration::seconds(secs as i64))
                .unwrap_or_default()
                .naive_utc(),
        };

        self.payments
            .retain(|p| p.time > period_start.timestamp() as u64)
    }

    pub fn sum_payments(&mut self) -> u64 {
        let now = Utc::now();
        self.clean_old_payments(now);
        self.payments.iter().map(|p| p.amt).sum()
    }

    pub fn budget_remaining(&self) -> u64 {
        let mut clone = self.clone();
        self.budget.saturating_sub(clone.sum_payments())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpendingConditions {
    SingleUse(SingleUseSpendingConditions),
    /// Require approval before sending a payment
    RequireApproval,
    Budget(BudgetedSpendingConditions),
}

impl Default for SpendingConditions {
    fn default() -> Self {
        Self::RequireApproval
    }
}
