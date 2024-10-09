package com.fedimintClientd.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class LightningCreateInvoiceResponse(
    val operationId: String,
    val invoice: String
)

@Serializable
data class LightningInvoiceForPubkeyTweakResponse(
    val operationId: String,
    val invoice: String
)

@Serializable
data class LightningPayResponse(
    val operationId: String,
    val paymentType: PaymentType,
    val contractId: String,
    val fee: Int
)

@Serializable
data class PaymentType(
    val internal: String,
)

@Serializable
sealed class LnPaymentResult {
    data class WaitingForPayment(val paymentRequest: String) : LnPaymentResult()
    object Canceled : LnPaymentResult()
}

@Serializable
data class LightningPaymentResponse(
    val state: LnReceiveState,
    val details: LnPaymentResult? = null
)

@Serializable
enum class LnReceiveState {
    Created,
    WaitingForPayment,
    Canceled,
    Funded,
    AwaitingFunds,
    Claimed
}

@Serializable
data class GatewayFees(
    @SerialName("base_msat")
    val baseMsat: Int,
    @SerialName("proportional_millionths")
    val proportionalMillionths: Int
)

@Serializable
data class GatewayInfo(
    val api: String,
    val fees: GatewayFees,
    @SerialName("gateway_id")
    val gatewayId: String,
    @SerialName("gateway_redeem_key")
    val gatewayRedeemKey: String,
    @SerialName("lightning_alias")
    val lightningAlias: String,
    @SerialName("mint_channel_id")
    val mintChannelId: Int,
    @SerialName("node_pub_key")
    val nodePubKey: String,
    @SerialName("route_hints")
    val routeHints: List<String>,
    @SerialName("supports_private_payments")
    val supportsPrivatePayments: Boolean
)

@Serializable
data class GatewayTTL(
    val nanos: Long,
    val secs: Int
)

@Serializable
data class Gateway(
    @SerialName("federation_id")
    val federationId: String,
    val info: GatewayInfo,
    val ttl: GatewayTTL,
    val vetted: Boolean
)
