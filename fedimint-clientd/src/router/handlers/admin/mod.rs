pub mod backup;
pub mod config;
pub mod discover_version;
pub mod federation_ids;
pub mod info;
pub mod join;
pub mod list_operations;
pub mod module;
pub mod restore;

use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_mint_client::MintClientModule;
use multimint::fedimint_wallet_client::WalletClientModule;

use self::info::FedimintClientInfo;

pub async fn _get_note_summary(client: &ClientHandleArc) -> anyhow::Result<FedimintClientInfo> {
    let mint_client = client.get_first_module::<MintClientModule>();
    let wallet_client = client.get_first_module::<WalletClientModule>();
    let summary = mint_client
        .get_wallet_summary(
            &mut client
                .db()
                .begin_transaction_nc()
                .await
                .to_ref_with_prefix_module_id(1),
        )
        .await;
    Ok(FedimintClientInfo {
        network: wallet_client.get_network().to_string(),
        meta: client.get_config().global.meta.clone(),
        total_amount_msat: summary.total_amount(),
        total_num_notes: summary.count_items(),
        denominations_msat: summary,
    })
}
