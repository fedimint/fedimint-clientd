use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

const CACHE_DURATION: Duration = Duration::from_secs(86_400); // 1 day

#[derive(Debug, Clone)]
struct Payment {
    time: Instant,
    amount: u64,
    destination: String, // Assuming destination is a string identifier
}

#[derive(Debug, Clone)]
pub struct PaymentsManager {
    payments: VecDeque<Payment>,
    max_amount: u64,
    daily_limit: u64,
    rate_limit: Duration, // New: Limit for frequency of payments
    max_destination_amounts: HashMap<String, u64>, // New: Max amounts per destination
}

impl PaymentsManager {
    pub fn new(max_amount: u64, daily_limit: u64, rate_limit_secs: u64) -> Self {
        PaymentsManager {
            payments: VecDeque::new(),
            max_amount,
            daily_limit,
            rate_limit: Duration::from_secs(rate_limit_secs),
            max_destination_amounts: HashMap::new(),
        }
    }

    pub fn add_payment(&mut self, amount: u64, destination: String) -> Result<(), String> {
        let now = Instant::now();

        // Check rate limit
        if let Some(last_payment) = self.payments.back() {
            if now.duration_since(last_payment.time) < self.rate_limit {
                return Err("Rate limit exceeded.".to_string());
            }
        }

        // Check max amount per destination
        if let Some(&max_amount) = self.max_destination_amounts.get(&destination) {
            if amount > max_amount {
                return Err("Destination max amount exceeded.".to_string());
            }
        }

        let payment = Payment {
            time: now,
            amount,
            destination,
        };
        self.payments.push_back(payment);
        Ok(())
    }

    fn clean_old_payments(&mut self) {
        let now = Instant::now();
        while let Some(payment) = self.payments.front() {
            if now.duration_since(payment.time) < CACHE_DURATION {
                break;
            }
            self.payments.pop_front();
        }
    }

    pub fn sum_payments(&mut self) -> u64 {
        self.clean_old_payments();
        self.payments.iter().map(|p| p.amount).sum()
    }

    pub fn check_payment_limits(&mut self, msats: u64) -> Option<String> {
        if self.max_amount > 0 && msats > self.max_amount * 1_000 {
            Some("Invoice amount too high.".to_string())
        } else if self.daily_limit > 0 && self.sum_payments() + msats > self.daily_limit * 1_000 {
            Some("Daily limit exceeded.".to_string())
        } else {
            None
        }
    }

    // New: Set maximum amount for a specific destination
    pub fn set_max_amount_for_destination(&mut self, destination: String, max_amount: u64) {
        self.max_destination_amounts.insert(destination, max_amount);
    }
}
