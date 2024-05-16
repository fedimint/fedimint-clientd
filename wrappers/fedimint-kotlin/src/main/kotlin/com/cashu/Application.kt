package com.cashu

import kotlinx.coroutines.runBlocking
import java.lang.Exception
import kotlin.system.exitProcess
import io.github.cdimascio.dotenv.dotenv

fun main() {
    val fedimintClient=buildFedimintClient()
    if(fedimintClient==null){
        exitProcess(500)
    }
    val mint = fedimintClient.MintModule()
    val ln = fedimintClient.LightningModule()
    val onchain = fedimintClient.OnChainModule()
    runBlocking {
        //        Admin
        val info = fedimintClient.info()
        println("""🚀🚀🚀$info""")

        val config = fedimintClient.config()
        println("🚀🚀🚀" + config)

        val version = fedimintClient.discoverVersion(1)
        println("🚀🚀🚀" + version)

        val federationId = fedimintClient.federationIds()
        println("🚀🚀🚀" + federationId)

        val join =
            fedimintClient.join("fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75")
        println("🚀🚀🚀" + join)

        val operations = fedimintClient.listOperations(10)
        println("🚀🚀🚀" + operations)

        //        Onchain

        val address = onchain.createDepositAddress(1000)
        println("🚀🚀🚀" + address)

        val withdraw = address?.let { onchain.withdraw(it.address, 1000) }
        println("🚀🚀🚀" + withdraw)

        //        Lightning

        val gateways = ln.listGateways()
        println("🚀🚀🚀" + gateways)
        if(gateways.isNotEmpty()){
            fedimintClient.activeGatewayId = gateways.first().info.gatewayId
        }

        val invoice = ln.createInvoice(1000, "Test")
        println("🚀🚀🚀" + invoice)

        val awaitInvoice = invoice?.let { ln.awaitInvoice(operationId = it.operationId) }
        println("🚀🚀🚀" + awaitInvoice)

        val pay = invoice?.let { ln.pay(paymentInfo = it.invoice) }
        println("🚀🚀🚀" + pay)

        val awaitPay = pay?.let { ln.awaitPay(operationId = it.operationId) }
        println("🚀🚀🚀" + awaitPay)

        //        Mint

        val spend = mint.spend(amountMsat = 3000, allowOverpay = true, timeout = 1000, includeInvite = false)
        println("🚀🚀🚀" + spend)

        val notes = spend?.notes?.let { mint.decodeNotes(it) }
        println("🚀🚀🚀" + notes)

        val encodedNotes = notes?.notesJson?.let { mint.encodeNotes(it) }
        println("🚀🚀🚀" + encodedNotes)

        val validate = spend?.notes?.let { mint.validate(it) }
        println("🚀🚀🚀" + validate)

        val reissue = spend?.notes?.let { mint.reissue(it) }
        println("🚀🚀🚀" + reissue)

        val split = spend?.notes?.let { mint.split(it) }
        println("🚀🚀🚀" + split)

    }
}

fun buildFedimintClient(): FedimintClient? {
    try {
        // Uncomment ignoreIfMissing to use the default values below
        val dotenv = dotenv {
            // Provide an absolute path to the .env file
            directory = "/absolute/path/to/file"
            // ignoreIfMalformed = true
            // ignoreIfMissing = true
        }

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
