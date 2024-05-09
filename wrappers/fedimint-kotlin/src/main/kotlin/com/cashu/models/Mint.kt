package com.cashu.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class NotesJson(
    @SerialName("federation_id_prefix")
    val federationIdPrefix: String,
    val notes: Map<String, List<Note>>
)

@Serializable
data class Note(
    val signature: String,
    val spendKey: String
)

@Serializable
data class MintSpendResponse(
    val operation: String,
    val notes: String
)

@Serializable
data class MintDecodeNotesResponse(
    val notesJson: NotesJson,
)