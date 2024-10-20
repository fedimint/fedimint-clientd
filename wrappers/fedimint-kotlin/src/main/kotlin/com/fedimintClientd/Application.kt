package com.fedimintClientd

import io.github.cdimascio.dotenv.Dotenv
import kotlinx.coroutines.runBlocking
import kotlin.system.exitProcess
import io.github.cdimascio.dotenv.dotenv

fun main() {
    try {
        val dotenv = dotenv {
            // ignoreIfMalformed = true
             ignoreIfMissing = true
        }

        val fedimintClient = buildFedimintClient(dotenv) ?: exitProcess(500)

        val mint = fedimintClient.MintModule()
        val ln = fedimintClient.LightningModule()
        val onchain = fedimintClient.OnChainModule()
        runBlocking {
            //        Admin

            logMethod("/v2/admin/info")
            val info = fedimintClient.info()
            if(info.second!=null){
                throw Exception(info.second)
            }
            logInputAndOutput(emptyMap<String, Any>(), info)

            logMethod("/v2/admin/config")
            val config = fedimintClient.config()
            if(config.second!=null){
                throw Exception(config.second)
            }
            logInputAndOutput(emptyMap<String, Any>(), config)

            logMethod("/v2/admin/discover-version")
            val version = fedimintClient.discoverVersion(1)
            if(version.second!=null){
                throw Exception(version.second)
            }
            logInputAndOutput(emptyMap<String, Any>(), version)

            logMethod("/v2/admin/federation-ids")
            val federationId = fedimintClient.federationIds()
            if(federationId.second!=null){
                throw Exception(federationId.second)
            }
            logInputAndOutput(emptyMap<String, Any>(), federationId)

            logMethod("/v2/admin/join")
            val inviteCode = dotenv["FEDIMINT_CLIENTD_INVITE_CODE"]
                ?: "fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75"
            val join =
                fedimintClient.join(inviteCode)
            if(join.second!=null){
                throw Exception(join.second)
            }
            logInputAndOutput(mapOf( "inviteCode" to inviteCode ), join)

            logMethod("/v2/admin/list-operations")
            val operations = fedimintClient.listOperations(10)
            if(operations.second!=null){
                throw Exception(operations.second)
            }
            logInputAndOutput(mapOf( "limit" to 10 ), operations)

            //        Onchain

            logMethod("/v2/onchain/deposit-address")
            val address = onchain.createDepositAddress(1000)
            if(address.second!=null){
                throw Exception(address.second)
            }
            logInputAndOutput(mapOf( "timeout" to 1000 ), address)

            logMethod("/v2/onchain/withdraw")
            val withdraw = address.first?.let { onchain.withdraw(it.address, 1000) }
            if(withdraw?.second!=null){
                throw Exception(withdraw.second)
            }
            logInputAndOutput(mapOf(
                "address" to address.first?.address,
                "amountSat" to 1000
            ), withdraw)

            //        Lightning

            logMethod("/v2/ln/list-gateways")
            val gateways = ln.listGateways()
            logInputAndOutput(emptyMap<String, Any>(), gateways)
            if (gateways.isNotEmpty()) {
                fedimintClient.activeGatewayId = gateways.first().info.gatewayId
            }

            logMethod("/v2/ln/invoice")
            val invoice = ln.createInvoice(1000, "Test")
            if(invoice.second!=null){
                throw Exception(invoice.second)
            }
            logInputAndOutput(mapOf(
                "amountMsat" to 1000,
                "description" to "Test"
            ), invoice)

            logMethod("/v2/ln/await-invoice")
            val awaitInvoice = invoice.first?.let { ln.awaitInvoice(operationId = it.operationId) }
            if(awaitInvoice?.second!=null){
                throw Exception(awaitInvoice.second)
            }
            logInputAndOutput(mapOf( "operationId" to invoice.first?.operationId ), awaitInvoice)

            logMethod("/v2/ln/pay")
            val pay = invoice.first?.let { ln.pay(paymentInfo = it.invoice) }
            if(pay?.second!=null){
                throw Exception(pay.second)
            }
            logInputAndOutput(mapOf( "paymentInfo" to invoice.first?.invoice ), pay)

            val awaitPay = pay?.first?.let { ln.awaitPay(operationId = it.operationId) }
            if(awaitPay?.second!=null){
                throw Exception(awaitPay.second)
            }
            logInputAndOutput(mapOf( "operationId" to pay?.first?.operationId ), awaitPay)

            //        Mint

            logMethod("/v2/mint/spend")
            val spend = mint.spend(amountMsat = 3000, allowOverpay = true, timeout = 1000, includeInvite = false)
            if(spend.second!=null){
                throw Exception(spend.second)
            }
            logInputAndOutput(mapOf(
                "amountMsat" to 3000,
                "allowOverpay" to true,
                "timeout" to 1000,
                "includeInvite" to false
            ), spend)

            logMethod("/v2/mint/decode-notes")
            val notes = spend.first?.notes?.let { mint.decodeNotes(it) }
            if(notes?.second!=null){
                throw Exception(notes.second)
            }
            logInputAndOutput(mapOf( "notes" to spend.first?.notes ), notes)

            logMethod("/v2/mint/encode-notes")
            val encodedNotes = notes?.first?.notesJson?.let { mint.encodeNotes(it) }
            if(encodedNotes?.second!=null){
                throw Exception(encodedNotes.second)
            }
            logInputAndOutput(mapOf( "notesJson" to notes?.first?.notesJson ), encodedNotes)

            logMethod("/v2/mint/validate")
            val validate = spend.first?.notes?.let { mint.validate(it) }
            if(validate?.second!=null){
                throw Exception(validate.second)
            }
            logInputAndOutput(mapOf( "notes" to spend.first?.notes ), validate)

            logMethod("/v2/mint/reissue")
            val reissue = spend.first?.notes?.let { mint.reissue(it) }
            if(reissue?.second!=null){
                throw Exception(reissue.second)
            }
            logInputAndOutput(mapOf( "notes" to spend.first?.notes ), reissue)

            logMethod("/v2/mint/split")
            val split = spend.first?.notes?.let { mint.split(it) }
            if(split?.second!=null){
                throw Exception(split.second)
            }
            logInputAndOutput(mapOf( "notes" to spend.first?.notes ), split)

            println("🚀Done: All Methods Tested Successfully🚀")
        }
    } catch (e: Exception) {
        println("Test Failed:: ${e.localizedMessage}")
    }
}

fun buildFedimintClient(dotenv: Dotenv): FedimintClient? {
    try {
        val baseUrl = dotenv["FEDIMINT_CLIENTD_BASE_URL"] ?: "http://127.0.0.1:3333"
        val password = dotenv["FEDIMINT_CLIENTD_PASSWORD"] ?: "password"
        val federationId = dotenv["FEDIMINT_CLIENTD_ACTIVE_FEDERATION_ID"]
            ?: "15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3"

        return FedimintClient(baseUrl = baseUrl, password = password, activeFederationId = federationId)
    } catch (e: Exception) {
        println("Test Failed:: ${e.localizedMessage}")
    }
    return null
}

fun logMethod(method: String) {
    println("--------------------")
    println("Method: $method")
}

fun logInputAndOutput(inputData: Any, output: Any?) {
    println("Input: $inputData")
    println("Output: $output")
    println("--------------------")
}
