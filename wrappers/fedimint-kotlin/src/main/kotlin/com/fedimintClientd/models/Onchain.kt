package com.fedimintClientd.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

sealed class OnchainAwaitDepositResponse(val status: Any) {
    data class Confirmed(val confirmed: AwaitDepositResponseConfirmed) : OnchainAwaitDepositResponse(confirmed)
    data class Error(val message: String) : OnchainAwaitDepositResponse(message)
}

@Serializable
data class OnchainWithdrawResponse(
    val fees_sat: Int,
    val txid: String
)

@Serializable
data class OnchainCreateAddressResponse(
    val address: String,
    val operationId: String
)

@Serializable
data class AwaitDepositResponseConfirmed(
    val btcTransaction: BTCTransaction,
    val outIdx: Int
)

@Serializable
data class BTCInput(
    @SerialName("previous_output")
    val previousOutput: String,
    @SerialName("script_sig")
    val scriptSig: String,
    val sequence: Int,
    val witness: List<String>
)

@Serializable
data class BTCOutput(
    val value: Long,
    @SerialName("script_pubkey")
    val scriptPubkey: String
)

@Serializable
data class BTCTransaction(
    val version: Int,
    @SerialName("lock_time")
    val lockTime: Int,
    val input: List<BTCInput>,
    val output: List<BTCOutput>
)
