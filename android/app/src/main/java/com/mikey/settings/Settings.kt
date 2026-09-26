package com.mikey.settings

import android.content.Context
import java.security.SecureRandom

/** Everything the app saves. Defaults live here and nowhere else. */
class Settings(context: Context) {
    private val prefs = context.getSharedPreferences("mikey", Context.MODE_PRIVATE)

    /** Random 128-bit id as 32 hex chars. Created on first use, then kept for good. */
    val deviceId: String
        get() = prefs.getString(KEY_DEVICE_ID, null)
            ?: newDeviceId().also { prefs.edit().putString(KEY_DEVICE_ID, it).apply() }

    /** PC address typed in for Wi-Fi testing (debug builds only). Null means connect over USB. */
    var manualPcAddress: String?
        get() = prefs.getString(KEY_MANUAL_PC_ADDRESS, null)
        set(value) = prefs.edit().putString(KEY_MANUAL_PC_ADDRESS, value?.trim()?.ifEmpty { null }).apply()

    private companion object {
        const val KEY_DEVICE_ID = "device.id"
        const val KEY_MANUAL_PC_ADDRESS = "pc.manualAddress"
    }
}

private fun newDeviceId(): String {
    val bytes = ByteArray(16).also { SecureRandom().nextBytes(it) }
    return bytes.joinToString("") { "%02x".format(it) }
}
