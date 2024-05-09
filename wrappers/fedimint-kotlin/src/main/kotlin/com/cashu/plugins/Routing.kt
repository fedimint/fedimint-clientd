package com.cashu.plugins

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.JsonPrimitive

fun Application.configureRouting() {

    routing {
        get("/") {
            call.respondText("Hello World!")
        }

    }
}
