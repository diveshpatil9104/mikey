package com.mikey.protocol

import org.json.JSONArray
import org.json.JSONObject
import java.nio.ByteBuffer

const val PROTO_VERSION = 1

fun helloPayload(deviceId: String, deviceName: String, level: Int): ByteArray =
    JSONObject()
        .put("proto", PROTO_VERSION)
        .put("device_id", deviceId)
        .put("device_name", deviceName)
        .put("level", level)
        .put("caps", JSONArray().put("audio"))
        .toString()
        .toByteArray()

class Welcome(val pcId: String, val pcName: String)

fun parseWelcome(payload: ByteArray): Welcome {
    val json = JSONObject(String(payload))
    return Welcome(json.getString("pc_id"), json.getString("pc_name"))
}

fun heartbeatPayload(sentAtUs: Long): ByteArray = ByteBuffer.allocate(Long.SIZE_BYTES).putLong(sentAtUs).array()

fun heartbeatSentAt(payload: ByteArray): Long = ByteBuffer.wrap(payload).long

fun byePayload(reason: String): ByteArray = JSONObject().put("reason", reason).toString().toByteArray()
