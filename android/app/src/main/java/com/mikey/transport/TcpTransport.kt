package com.mikey.transport

import java.io.IOException
import java.net.InetSocketAddress
import java.net.Socket

/** The PC listens on this port on every level. */
private const val PC_PORT = 7653

/** No frame from the PC for this long means the link is dead. */
private const val LINK_TIMEOUT_MS = 6_000

/**
 * A plain TCP link to the PC. Phase 1 has two: Level 1 reaches the PC through the
 * `adb reverse` tunnel on localhost, and a typed-in address (debug only) is Level 4.
 */
class TcpTransport private constructor(
    private val host: String,
    val level: Int,
    private val connectTimeoutMs: Int,
) {
    fun open(): Socket {
        val socket = Socket()
        try {
            socket.tcpNoDelay = true
            socket.soTimeout = LINK_TIMEOUT_MS
            socket.connect(InetSocketAddress(host, PC_PORT), connectTimeoutMs)
        } catch (e: IOException) {
            socket.close()
            throw e
        }
        return socket
    }

    companion object {
        fun adb() = TcpTransport("127.0.0.1", level = 1, connectTimeoutMs = 300)

        fun manual(address: String) = TcpTransport(address, level = 4, connectTimeoutMs = 2_000)
    }
}
