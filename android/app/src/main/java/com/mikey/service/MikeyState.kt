package com.mikey.service

/** What the screen shows. MikeyService owns it; the UI only reads it. */
data class MikeyState(
    val micOn: Boolean = false,
    val connected: Boolean = false,
)
