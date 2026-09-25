package com.mikey.protocol

/** Frame type byte (vibe/architecture/wire-protocol.md). Only the types Phase 1 uses. */
object FrameType {
    const val HELLO = 0x00
    const val AUDIO = 0x01
    const val HEARTBEAT = 0x03
    const val BYE = 0x05
    const val WELCOME = 0x10
}
