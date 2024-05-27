use lightning_invoice::Bolt11Invoice;
use redb::{TableDefinition, TypeName, Value};
use serde::{Deserialize, Serialize};

pub const PAYMENTS_TABLE: TableDefinition<&str, Payment> = TableDefinition::new("payments");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub time: u64,
    pub amount: u64,
    pub invoice: Bolt11Invoice,
}

impl Payment {
    pub fn new(time: u64, amount: u64, invoice: Bolt11Invoice) -> Self {
        Self {
            time,
            amount,
            invoice,
        }
    }
}

impl Value for Payment {
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
        TypeName::new("Payment")
    }
}
