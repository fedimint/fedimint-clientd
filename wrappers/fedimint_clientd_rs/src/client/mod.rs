mod admin;
mod base;
mod http;
mod lightning;
mod mint;
mod onchain;

pub struct FedimintClient {
    base_url: String,
    password: String,
    active_federation_id: String,
    active_gateway_id: String,
    built: bool,
}
