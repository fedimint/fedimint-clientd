package com.fedimintClientd

import com.fedimintClientd.models.*
import com.google.gson.Gson
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.cio.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.client.plugins.auth.*
import io.ktor.client.plugins.auth.providers.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.serialization.gson.*
import kotlinx.serialization.json.Json
import kotlinx.serialization.*
import kotlinx.serialization.json.*

class FedimintClient(
    var baseUrl: String,
    val password: String,
    val activeFederationId: String,
    var activeGatewayId: String? = null
) {
    init {
        baseUrl = "$baseUrl/v2/"
    }

    private val client = HttpClient(CIO) {
        install(ContentNegotiation) {
            gson()
        }
        install(Auth) {
            bearer {
                loadTokens {
                    BearerTokens(password, "")
                }
            }
        }
    }

    suspend fun _get(endpoint: String): Pair<String?, String?> {
        try {
            val response = client.get("${baseUrl}${endpoint}")
            return Pair(response.body(), null)
        } catch (e: Exception) {
            return Pair(null, e.localizedMessage)
        }
    }

    suspend fun _post(endpoint: String, data: String? = null): Pair<String?, String?> {
        try {
            val response = client.post("${baseUrl}${endpoint}") {
                contentType(ContentType.Application.Json)
                setBody(data)
            }
            return if (response.status.value == 200) {
                Pair(response.bodyAsText(), null)
            } else {
                Pair(null, "${response.status}")
            }
        } catch (e: Exception) {
            return Pair(null, e.localizedMessage)
        }
    }

    suspend fun _postWithFederationId(
        endpoint: String,
        federationId: String? = null,
        data: Map<String, Any> = emptyMap()
    ): Pair<String?, String?> {
        try {
            val data = data.toMutableMap()
            data["federationId"] = federationId ?: this.activeFederationId
            val body = Gson().toJson(data)
            val response = this._post(endpoint, body)
            return response
        } catch (e: Exception) {
            return Pair(null, e.localizedMessage)
        }
    }

    suspend fun _postWithGatewayIdAndFederationId(
        endpoint: String,
        federationId: String? = null,
        gatewayId: String? = null,
        data: Map<String, Any?> = emptyMap()
    ): Pair<String?, String?> {
        try {
            val data = data.toMutableMap()
            data["federationId"] = federationId ?: this.activeFederationId
            data["gatewayId"] = gatewayId ?: this.activeGatewayId ?: throw Exception("Must set Active Gateway ID!")
            val body = Gson().toJson(data)
            val response = this._post(endpoint, body)
            return response
        } catch (e: Exception) {
            return Pair(null, e.localizedMessage)
        }
    }

    suspend fun info(): Pair<String?, String?> {
        return this._get("admin/info")
    }

    suspend fun config(): Pair<String?, String?> {
        return this._get("admin/config")
    }

    suspend fun discoverVersion(threshold: Int): Pair<String?, String?> {
        val data = mutableMapOf<String, Int>("inviteCode" to threshold)
        return this._post("admin/discover-version", data = Json.encodeToString(data))
    }

    suspend fun federationIds(): Pair<String?, String?> {
        return this._get("admin/federation-ids")
    }

    suspend fun join(inviteCode: String, useManualSecret: Boolean = false): Pair<String?, String?> {
        val data = buildJsonObject {
            put("useManualSecret", useManualSecret)
            put("inviteCode", inviteCode)
        }
        return _post("admin/join", data = data.toString())
    }

    suspend fun listOperations(limit: Int): Pair<String?, String?> {
        return this._postWithFederationId("admin/list-operations", data = mapOf("limit" to limit))
    }

    inner class MintModule {
        private val json = Json { ignoreUnknownKeys = true }

        suspend fun spend(
            amountMsat: Int,
            allowOverpay: Boolean,
            timeout: Int,
            includeInvite: Boolean,
            federationId: String? = null
        ): Pair<MintSpendResponse?, String?> {
            val mintSpendRequest = mapOf(
                "amountMsat" to amountMsat,
                "allowOverpay" to allowOverpay,
                "timeout" to timeout,
                "includeInvite" to includeInvite,
            )
            val res = _postWithFederationId(
                "mint/spend",
                federationId = federationId,
                data = mintSpendRequest
            )
            return if (res.first != null) {
                Pair(json.decodeFromString<MintSpendResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }

        suspend fun decodeNotes(notes: String, federationId: String? = null): Pair<MintDecodeNotesResponse?, String?> {
            val res = _postWithFederationId(
                "mint/decode-notes",
                federationId = federationId,
                data = mapOf("notes" to notes)
            )

            return if (res.first != null) {
                Pair(json.decodeFromString<MintDecodeNotesResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }

        suspend fun encodeNotes(notes: NotesJson, federationId: String? = null): Pair<String?, String?> {
            return _postWithFederationId(
                "mint/encode-notes",
                federationId = federationId,
                data = mapOf("notesJsonStr" to json.encodeToString(notes))
            )
        }

        suspend fun validate(notes: String, federationId: String? = null): Pair<String?, String?> {
            return _postWithFederationId(
                "mint/validate",
                federationId = federationId,
                data = mapOf("notes" to notes)
            )
        }

        suspend fun combine(notesVec: List<String>, federationId: String? = null): Pair<String?, String?> {
            return _postWithFederationId(
                "mint/combine",
                federationId = federationId,
                data = mapOf("notesVec" to notesVec)
            )
        }

        suspend fun reissue(notes: String, federationId: String? = null): Pair<String?, String?> {
            return _postWithFederationId(
                "mint/reissue",
                federationId = federationId,
                data = mapOf("notes" to notes)
            )
        }

        suspend fun split(notes: String, federationId: String? = null): Pair<String?, String?> {
            return _postWithFederationId(
                "mint/split",
                federationId = federationId,
                data = mapOf("notes" to notes)
            )
        }
    }

    inner class LightningModule {
        private val json = Json { ignoreUnknownKeys = true }

        suspend fun listGateways(): List<Gateway> {
            try {
                val res = _postWithFederationId(
                    "ln/list-gateways",
                )
                if (res.first != null) {
                    return json.decodeFromString<List<Gateway>>(res.first!!)
                }
            } catch (e: Exception) {
                println(e.localizedMessage)
            }
            return emptyList()
        }

        suspend fun createInvoice(
            amountMsat: Int,
            description: String,
            expiryTime: Int? = null,
            federationId: String? = null,
            gatewayId: String? = null
        ): Pair<LightningCreateInvoiceResponse?, String?> {
            val request = mutableMapOf<String, Any>(
                "amountMsat" to amountMsat,
                "description" to description,
            )
            if (expiryTime != null) {
                request["expiryTime"] = expiryTime
            }
            val res = _postWithGatewayIdAndFederationId(
                "ln/invoice",
                federationId = federationId,
                gatewayId = gatewayId,
                data = request
            )
            return if (res.first != null) {
                Pair(json.decodeFromString<LightningCreateInvoiceResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }

        suspend fun awaitInvoice(
            operationId: String,
            federationId: String? = null,
        ): Pair<LightningPaymentResponse?, String?> {
            val request = mutableMapOf<String, Any>(
                "operationId" to operationId,
            )
            val res = _postWithGatewayIdAndFederationId(
                "ln/await-invoice",
                federationId = federationId,
                data = request
            )

            return if (res.first != null) {
                Pair(json.decodeFromString<LightningPaymentResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }

        suspend fun pay(
            amountMsat: Int? = null,
            paymentInfo: String,
            lightningUrlComment: String? = null,
            federationId: String? = null,
            gatewayId: String? = null
        ): Pair<LightningPayResponse?, String?> {
            val request = mutableMapOf<String, Any?>(
                "amountMsat" to amountMsat,
                "lightningUrlComment" to lightningUrlComment,
                "paymentInfo" to paymentInfo,
            )

            val res = _postWithGatewayIdAndFederationId(
                "ln/pay",
                federationId = federationId,
                gatewayId = gatewayId,
                data = request
            )
            return if (res.first != null) {
                Pair(json.decodeFromString<LightningPayResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }

        suspend fun awaitPay(
            operationId: String,
            federationId: String? = null,
        ): Pair<String?, String?> {
            val request = mutableMapOf<String, Any>(
                "operationId" to operationId,
            )
            return _postWithGatewayIdAndFederationId(
                "ln/await-pay",
                federationId = federationId,
                data = request
            )
        }

        suspend fun createInvoiceForPubkeyTweak(
            tweak: Int,
            pubkey: String,
            amountMsat: Int,
            description: String,
            expiryTime: Int? = null,
            federationId: String? = null,
            gatewayId: String? = null
        ): Pair<LightningInvoiceForPubkeyTweakResponse?, String?> {
            val request = mutableMapOf<String, Any>(
                "tweak" to tweak,
                "externalPubkey" to pubkey,
                "amountMsat" to amountMsat,
                "description" to description,
            )
            if (expiryTime != null) {
                request["expiryTime"] = expiryTime
            }
            val res = _postWithGatewayIdAndFederationId(
                "ln/invoice-external-pubkey-tweaked",
                federationId = federationId,
                gatewayId = gatewayId,
                data = request
            )

            return if (res.first != null) {
                Pair(json.decodeFromString<LightningInvoiceForPubkeyTweakResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)

            }
        }

        suspend fun claimPubkeyTweakReceives(
            privateKey: String,
            tweaks: List<String>,
            federationId: String? = null,
        ): Pair<LightningPaymentResponse?, String?> {
            val request = mapOf(
                "tweaks" to tweaks,
                "privateKey" to privateKey,
            )

            val res = _postWithFederationId(
                "ln/claim-external-receive-tweaked",
                federationId = federationId,
                data = request
            )

            return if (res.first != null) {
                Pair(json.decodeFromString<LightningPaymentResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }
    }

    inner class OnChainModule {
        private val json = Json { ignoreUnknownKeys = true }

        suspend fun createDepositAddress(
            timeout: Int,
            federationId: String? = null
        ): Pair<OnchainCreateAddressResponse?, String?> {
            val res = _postWithFederationId(
                "onchain/deposit-address",
                federationId = federationId,
                data = mapOf("timeout" to timeout)
            )
            return if (res.first != null) {
                Pair(json.decodeFromString<OnchainCreateAddressResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }

        suspend fun awaitDeposit(
            operationId: String,
            federationId: String? = null
        ): Pair<OnchainAwaitDepositResponse?, String?> {
            val res = _postWithFederationId(
                "onchain/await-deposit",
                federationId = federationId,
                data = mapOf("operationId" to operationId)
            )

            return if (res.first != null) {
                Pair(json.decodeFromString<OnchainAwaitDepositResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }

        suspend fun withdraw(
            address: String,
            amountSat: Int?,
            withdrawAllSats: Boolean = false,
            federationId: String? = null
        ): Pair<OnchainWithdrawResponse?, String?> {
            var amnt = amountSat.toString()
            if (withdrawAllSats && amountSat == null) {
                amnt = "all"
            }

            val res = _postWithFederationId(
                "onchain/withdraw",
                federationId = federationId,
                data = mapOf("address" to address, "amountSat" to amnt)
            )

            return if (res.first != null) {
                Pair(json.decodeFromString<OnchainWithdrawResponse>(res.first!!), null)
            } else {
                Pair(null, res.second)
            }
        }
    }
}
