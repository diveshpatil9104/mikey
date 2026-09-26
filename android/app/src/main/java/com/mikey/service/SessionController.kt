package com.mikey.service

import android.content.Context
import android.os.Build
import android.os.SystemClock
import android.util.Log
import com.mikey.media.AudioCapture
import com.mikey.media.AudioFrame
import com.mikey.protocol.FrameType
import com.mikey.protocol.MediaHeader
import com.mikey.protocol.byePayload
import com.mikey.protocol.heartbeatPayload
import com.mikey.protocol.helloPayload
import com.mikey.protocol.parseWelcome
import com.mikey.protocol.readFrame
import com.mikey.protocol.writeFrame
import com.mikey.protocol.writeMediaFrame
import com.mikey.settings.Settings
import com.mikey.transport.TcpTransport
import org.json.JSONException
import java.io.BufferedInputStream
import java.io.BufferedOutputStream
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.IOException
import java.net.ProtocolException
import java.net.Socket
import java.util.concurrent.ArrayBlockingQueue
import java.util.concurrent.TimeUnit

/**
 * Streams the mic to the PC: connect, handshake, send audio and heartbeats, and reconnect with
 * backoff when the link drops. The network runs on its own thread; the capture thread only drops
 * frames into a small queue, so recording never waits on the network.
 *
 * [onLink] gets the connected level (1 = USB, 4 = Wi-Fi), or null when not connected.
 * It is called on the session thread.
 */
class SessionController(context: Context, private val onLink: (level: Int?) -> Unit) {
    private val settings = Settings(context)
    private val frames = ArrayBlockingQueue<AudioFrame>(QUEUE_FRAMES)
    private val capture = AudioCapture(context) { frames.offerDroppingOldest(it) }
    private val thread = Thread(::sessionLoop, "mikey-session")

    @Volatile private var running = false

    fun start() {
        running = true
        capture.start()
        thread.start()
    }

    /** Releases the mic at once. The session thread then says BYE and closes on its own. */
    fun stop() {
        running = false
        capture.stop()
        thread.interrupt()
    }

    private fun sessionLoop() {
        var failures = 0
        while (running) {
            // Read on every attempt, so a newly typed address is used on the next connect.
            val transport = settings.manualPcAddress?.let { TcpTransport.manual(it) } ?: TcpTransport.adb()
            try {
                transport.open().use { socket ->
                    val output = DataOutputStream(BufferedOutputStream(socket.getOutputStream()))
                    val input = DataInputStream(BufferedInputStream(socket.getInputStream()))
                    handshake(output, input, transport.level)
                    failures = 0
                    stream(socket, input, output, transport.level)
                }
            } catch (e: IOException) {
                Log.i(TAG, "No link to the PC: $e")
            }
            if (running) pause(reconnectDelayMs(failures++))
        }
    }

    private fun handshake(output: DataOutputStream, input: DataInputStream, level: Int) {
        output.writeFrame(FrameType.HELLO, helloPayload(settings.deviceId, Build.MODEL, level))
        output.flush()
        while (true) {
            val frame = input.readFrame()
            if (frame.type != FrameType.WELCOME) continue
            val welcome = try {
                parseWelcome(frame.payload)
            } catch (e: JSONException) {
                throw ProtocolException("Bad WELCOME: ${e.message}")
            }
            Log.i(TAG, "Connected to ${welcome.pcName} on level $level")
            return
        }
    }

    /** Sends audio and heartbeats until stopped (then says BYE), or throws when the link fails. */
    private fun stream(socket: Socket, input: DataInputStream, output: DataOutputStream, level: Int) {
        frames.clear() // Audio queued while we were offline is too old to play now.
        if (running) onLink(level)
        Thread({ receive(input, socket) }, "mikey-receive").start()
        try {
            var lastHeartbeatMs = 0L
            while (running) {
                val frame = try {
                    frames.poll(POLL_MS, TimeUnit.MILLISECONDS)
                } catch (e: InterruptedException) {
                    null
                }
                if (frame != null) {
                    output.writeMediaFrame(
                        FrameType.AUDIO,
                        frame.seq,
                        frame.captureTimeUs,
                        MediaHeader.CODEC_PCM_S16LE,
                        frame.pcm,
                        0,
                        frame.pcm.size,
                    )
                }
                val nowMs = SystemClock.elapsedRealtime()
                if (nowMs - lastHeartbeatMs >= HEARTBEAT_MS) {
                    output.writeFrame(FrameType.HEARTBEAT, heartbeatPayload(SystemClock.elapsedRealtimeNanos() / 1000))
                    lastHeartbeatMs = nowMs
                }
                output.flush()
            }
            sayBye(output)
        } finally {
            onLink(null)
            socket.close()
        }
    }

    /** Any frame from the PC proves the link is alive. Six silent seconds or a BYE end it. */
    private fun receive(input: DataInputStream, socket: Socket) {
        try {
            do {
                val frame = input.readFrame()
            } while (frame.type != FrameType.BYE)
        } catch (e: IOException) {
            // Timed out, dropped, or closed by the sender.
        } finally {
            socket.close() // Makes the sender's next write fail, so it reconnects.
        }
    }

    private fun sayBye(output: DataOutputStream) {
        try {
            output.writeFrame(FrameType.BYE, byePayload("stop"))
            output.flush()
        } catch (e: IOException) {
            // Best effort: the PC also notices the socket closing.
        }
    }

    private fun pause(ms: Long) {
        try {
            Thread.sleep(ms)
        } catch (e: InterruptedException) {
            // stop() wakes us so we can exit.
        }
    }

    private companion object {
        const val TAG = "SessionController"

        /** 200 ms of audio. Anything older would reach the PC too late to be played. */
        const val QUEUE_FRAMES = 20
        const val HEARTBEAT_MS = 2_000L
        const val POLL_MS = 100L
    }
}

/** Reconnect waits 0.5 s, 1 s, 2 s, 4 s, then 5 s from then on. */
internal fun reconnectDelayMs(failures: Int): Long = minOf(500L shl failures.coerceAtMost(4), 5_000L)

/** Never blocks: when the queue is full, the oldest item makes room. Fresh audio beats complete audio. */
internal fun <T> ArrayBlockingQueue<T>.offerDroppingOldest(item: T) {
    while (!offer(item)) poll()
}
