use fedimint_client::ClientArc;
use fedimint_mint_client::MintClientModule;
use fedimint_wallet_client::WalletClientModule;

use crate::types::fedimint::InfoResponse;

pub async fn get_note_summary(client: &ClientArc) -> anyhow::Result<InfoResponse> {
    let mint_client = client.get_first_module::<MintClientModule>();
    let wallet_client = client.get_first_module::<WalletClientModule>();
    let summary =
        mint_client
            .get_wallet_summary(
                &mut client
                    .db()
                    .begin_transaction_nc()
                    .await
                    .to_ref_with_prefix_module_id(1),
            )
            .await;
    Ok(InfoResponse {
        federation_id: client.federation_id(),
        network: wallet_client.get_network().to_string(),
        meta: client.get_config().global.meta.clone(),
        total_amount_msat: summary.total_amount(),
        total_num_notes: summary.count_items(),
        denominations_msat: summary,
    })
}
