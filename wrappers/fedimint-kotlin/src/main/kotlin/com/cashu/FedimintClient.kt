package com.cashu

import com.cashu.models.*
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
        baseUrl= "$baseUrl/v2/"
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

    suspend fun _get(endpoint: String): String? {
        try {
            println("🐛🐛🐛$endpoint🐛🐛🐛")
            val response = client.get("${baseUrl}${endpoint}")
            println("🐛🐛🐛$endpoint: ${response.status}🐛🐛🐛")
            println("🐛🐛🐛${response.bodyAsText()}🐛🐛🐛")
            return response.body()
        } catch (e: Exception) {
            println("🐛🐛🐛$endpoint: ${e.localizedMessage}🐛🐛🐛")
        }
        return null
    }

    suspend fun _post(endpoint: String, data: String? = null): String? {
        try {
            println("🐛🐛🐛$endpoint🐛🐛🐛")
            val response = client.post("${baseUrl}${endpoint}") {
                contentType(ContentType.Application.Json)
                setBody(data)
            }
            println("🐛🐛🐛$endpoint: ${response.status}🐛🐛🐛")
            println("🐛🐛🐛${response.bodyAsText()}🐛🐛🐛")
            if (response.status.value == 200) {
                return response.bodyAsText()
            }
        } catch (e: Exception) {
            println("🐛🐛🐛Error: ${e.localizedMessage ?: "Error Processing Request"}🐛🐛🐛")
        }
        return null
    }

    suspend fun _postWithFederationId(
        endpoint: String,
        federationId: String? = null,
        data: Map<String, Any> = emptyMap()
    ): String? {
        try {
            val data = data.toMutableMap()
            data["federationId"] = federationId ?: this.activeFederationId
            val body = Gson().toJson(data)
            val response = this._post(endpoint, body)
            return response
        } catch (e: Exception) {
            println("🐛🐛🐛Error: ${e.localizedMessage}🐛🐛🐛")
        }
        return null
    }

    suspend fun _postWithGatewayIdAndFederationId(
        endpoint: String,
        federationId: String? = null,
        gatewayId: String? = null,
        data: Map<String, Any?> = emptyMap()
    ): String? {
        try {
            val data = data.toMutableMap()
            data["federationId"] = federationId ?: this.activeFederationId
            data["gatewayId"] = gatewayId ?: this.activeGatewayId ?: throw Exception("Must set Active Gateway ID!")
            val body = Gson().toJson(data)
            val response = this._post(endpoint, body)
            return response
        } catch (e: Exception) {
            println("🐛🐛🐛$endpoint: ${e.localizedMessage}🐛🐛🐛")
        }
        return null
    }

    suspend fun info(): String? {
        return this._get("admin/info")
    }

    suspend fun config(): String? {
        return this._get("admin/config")
    }

    suspend fun discoverVersion(threshold: Int): String? {
        val data = mutableMapOf<String, Int>("inviteCode" to threshold)
        return this._post("admin/discover-version", data = Json.encodeToString(data))?.toString()
    }

    suspend fun federationIds(): String? {
        return this._get("admin/federation-ids")
    }

    suspend fun join(inviteCode: String, useManualSecret: Boolean = false): String? {
        val data = buildJsonObject {
            put("useManualSecret", useManualSecret)
            put("inviteCode", inviteCode)
        }
        return _post("admin/join", data = data.toString()).toString()
    }

    suspend fun listOperations(limit: Int): String? {
        return this._postWithFederationId("admin/list-operations", data = mapOf("limit" to limit)).toString()
    }

    inner class MintModule {
        suspend fun spend(
            amountMsat: Int,
            allowOverpay: Boolean,
            timeout: Int,
            includeInvite: Boolean,
            federationId: String? = null
        ): MintSpendResponse? {
            val mintSpendRequest = mapOf(
                "amountMsat" to amountMsat,
                "allowOverpay" to allowOverpay,
                "timeout" to timeout,
                "includeInvite" to includeInvite,
            )
            return _postWithFederationId(
                "mint/spend",
                federationId = federationId,
                data = mintSpendRequest
            ) as MintSpendResponse?
        }

        suspend fun decodeNotes(notes: String, federationId: String? = null): MintDecodeNotesResponse? {
            return _postWithFederationId(
                "mint/decode-notes",
                federationId = federationId,
                data = mapOf("notes" to notes)
            ) as MintDecodeNotesResponse?
        }

        suspend fun encodeNotes(notes: NotesJson, federationId: String? = null): String? {
            return _postWithFederationId(
                "mint/encode-notes",
                federationId = federationId,
                data = mapOf("notesJsonStr" to notes)
            ) as String?
        }

        suspend fun validate(notes: String, federationId: String? = null): String? {
            return _postWithFederationId(
                "mint/validate",
                federationId = federationId,
                data = mapOf("notes" to notes)
            ) as String?
        }

        suspend fun combine(notesVec: List<String>, federationId: String? = null): String? {
            return _postWithFederationId(
                "mint/combine",
                federationId = federationId,
                data = mapOf("notesVec" to notesVec)
            ) as String?
        }

        suspend fun reissue(notes: String, federationId: String? = null): String? {
            return _postWithFederationId(
                "mint/reissue",
                federationId = federationId,
                data = mapOf("notes" to notes)
            ) as String?
        }

        suspend fun split(notes: String, federationId: String? = null): String? {
            return _postWithFederationId(
                "mint/split",
                federationId = federationId,
                data = mapOf("notes" to notes)
            ) as String?
        }
    }

    inner class LightningModule {
        private val json = Json { ignoreUnknownKeys = true }

        suspend fun listGateways(): List<Gateway> {
            try {
                val res = _postWithFederationId(
                    "ln/list-gateways",
                ) as String?
                if (res != null) {
                    return json.decodeFromString<List<Gateway>>(res)
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
        ): LightningCreateInvoiceResponse? {
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
            if (res != null) {
                return json.decodeFromString<LightningCreateInvoiceResponse>(res)
            }
            return null
        }

        suspend fun awaitInvoice(
            operationId: String,
            federationId: String? = null,
        ): LightningPaymentResponse? {
            val request = mutableMapOf<String, Any>(
                "operationId" to operationId,
            )
            val res = _postWithGatewayIdAndFederationId(
                "ln/await-invoice",
                federationId = federationId,
                data = request
            )

            if (res != null) {
                return json.decodeFromString<LightningPaymentResponse>(res)
            }
            return null
        }

        suspend fun pay(
            amountMsat: Int? = null,
            paymentInfo: String,
            lightningUrlComment: String? = null,
            federationId: String? = null,
            gatewayId: String? = null
        ): LightningPayResponse? {
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
            if (res != null) {
                return json.decodeFromString<LightningPayResponse>(res)
            }
            return null
        }

        suspend fun awaitPay(
            operationId: String,
            federationId: String? = null,
        ): String? {
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
        ): LightningInvoiceForPubkeyTweakResponse? {
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

            if (res != null) {
                return json.decodeFromString<LightningInvoiceForPubkeyTweakResponse>(res)
            }
            return null
        }

        suspend fun claimPubkeyTweakReceives(
            privateKey: String,
            tweaks: List<String>,
            federationId: String? = null,
        ): LightningPaymentResponse? {
            val request = mapOf(
                "tweaks" to tweaks,
                "privateKey" to privateKey,
            )

            val res = _postWithFederationId(
                "ln/claim-external-receive-tweaked",
                federationId = federationId,
                data = request
            )

            if (res != null) {
                return json.decodeFromString<LightningPaymentResponse>(res)
            }
            return null
        }
    }

    inner class OnChainModule {
        private val json = Json { ignoreUnknownKeys = true }

        suspend fun createDepositAddress(timeout: Int, federationId: String? = null): OnchainCreateAddressResponse? {
            val res = _postWithFederationId(
                "onchain/deposit-address",
                federationId = federationId,
                data = mapOf("timeout" to timeout)
            )
            if (res != null) {
                return json.decodeFromString<OnchainCreateAddressResponse>(res)
            }
            return null
        }

        suspend fun awaitDeposit(operationId: String, federationId: String? = null): OnchainAwaitDepositResponse? {
            val res = _postWithFederationId(
                "onchain/await-deposit",
                federationId = federationId,
                data = mapOf("operationId" to operationId)
            )

            if (res != null) {
                return json.decodeFromString<OnchainAwaitDepositResponse>(res)
            }
            return null
        }

        suspend fun withdraw(address: String, amountSat: Int?, withdrawAllSats:Boolean=false, federationId: String? = null): OnchainWithdrawResponse? {
            var amnt=amountSat.toString()
            if(withdrawAllSats && amountSat==null){
                amnt="all"
            }

            val res = _postWithFederationId(
                "onchain/withdraw",
                federationId = federationId,
                data = mapOf("address" to address, "amountSat" to amnt)
            )

            if (res != null) {
                return json.decodeFromString<OnchainWithdrawResponse>(res)
            }
            return null
        }
    }
}
