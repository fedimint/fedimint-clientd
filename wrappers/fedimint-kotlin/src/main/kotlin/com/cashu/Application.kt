package com.cashu

import io.github.cdimascio.dotenv.Dotenv
import kotlinx.coroutines.runBlocking
import java.lang.Exception
import kotlin.system.exitProcess
import io.github.cdimascio.dotenv.dotenv

fun main() {
    try {
        val dotenv = dotenv {
            // Provide an absolute path to the .env file
            directory = "/Absolute/path/to/file/.env"
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
            logInputAndOutput({}, info)

            logMethod("/v2/admin/config")
            val config = fedimintClient.config()
            logInputAndOutput({}, config)

            logMethod("/v2/admin/discover-version")
            val version = fedimintClient.discoverVersion(1)
            logInputAndOutput({}, version)

            logMethod("/v2/admin/federation-ids")
            val federationId = fedimintClient.federationIds()
            logInputAndOutput({}, federationId)

            logMethod("/v2/admin/join")
            val inviteCode = dotenv["FEDIMINT_CLIENTD_INVITE_CODE"]
                ?: "fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75"
            val join =
                fedimintClient.join(inviteCode)
            logInputAndOutput({ "inviteCode" to inviteCode }, join)

            logMethod("/v2/admin/list-operations")
            val operations = fedimintClient.listOperations(10)
            logInputAndOutput({ "limit" to 10 }, operations)

            //        Onchain

            logMethod("/v2/onchain/deposit-address")
            val address = onchain.createDepositAddress(1000)
            logInputAndOutput({ "timeout" to 1000 }, address)

            logMethod("/v2/onchain/withdraw")
            val withdraw = address?.let { onchain.withdraw(it.address, 1000) }
            logInputAndOutput({
                "address" to address?.address
                "amountSat" to 1000
            }, withdraw)

            //        Lightning

            logMethod("/v2/ln/list-gateways")
            val gateways = ln.listGateways()
            logInputAndOutput({}, gateways)
            if (gateways.isNotEmpty()) {
                fedimintClient.activeGatewayId = gateways.first().info.gatewayId
            }

            logMethod("/v2/ln/invoice")
            val invoice = ln.createInvoice(1000, "Test")
            logInputAndOutput({
                "amountMsat" to 1000
                "description" to "Test"
            }, invoice)

            logMethod("/v2/ln/await-invoice")
            val awaitInvoice = invoice?.let { ln.awaitInvoice(operationId = it.operationId) }
            logInputAndOutput({ "operationId" to invoice?.operationId }, awaitInvoice)

            logMethod("/v2/ln/pay")
            val pay = invoice?.let { ln.pay(paymentInfo = it.invoice) }
            logInputAndOutput({ "paymentInfo" to invoice?.invoice }, pay)

            val awaitPay = pay?.let { ln.awaitPay(operationId = it.operationId) }
            logInputAndOutput({ "operationId" to pay?.operationId }, awaitPay)

            //        Mint

            logMethod("/v2/mint/spend")
            val spend = mint.spend(amountMsat = 3000, allowOverpay = true, timeout = 1000, includeInvite = false)
            logInputAndOutput({
                "amountMsat" to 3000
                "allowOverpay" to true
                "timeout" to 1000
                "includeInvite" to false
            }, spend)

            logMethod("/v2/mint/decode-notes")
            val notes = spend?.notes?.let { mint.decodeNotes(it) }
            logInputAndOutput({ "notes" to spend?.notes }, notes)

            logMethod("/v2/mint/encode-notes")
            val encodedNotes = notes?.notesJson?.let { mint.encodeNotes(it) }
            logInputAndOutput({ "notesJson" to notes?.notesJson }, encodedNotes)

            logMethod("/v2/mint/validate")
            val validate = spend?.notes?.let { mint.validate(it) }
            logInputAndOutput({ "notes" to spend?.notes }, validate)

            logMethod("/v2/mint/reissue")
            val reissue = spend?.notes?.let { mint.reissue(it) }
            logInputAndOutput({ "notes" to spend?.notes }, reissue)

            logMethod("/v2/mint/split")
            val split = spend?.notes?.let { mint.split(it) }
            logInputAndOutput({ "notes" to spend?.notes }, split)

            println("🚀Done: All Methods Tested Successfully🚀")
        }
    } catch (e: Exception) {
        println("🐛🐛🐛${e.localizedMessage}🐛🐛🐛")
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
        println("🐛🐛🐛${e.localizedMessage}🐛🐛🐛")
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
