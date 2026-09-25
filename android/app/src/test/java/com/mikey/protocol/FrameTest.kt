package com.mikey.protocol

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Test
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.io.DataOutputStream
import java.net.ProtocolException

class FrameTest {

    @Test
    fun frameIsTypeThenBigEndianLengthThenPayload() {
        val encoded = encode { writeFrame(FrameType.HELLO, bytesOf(7, 8, 9)) }

        assertArrayEquals(bytesOf(0x00, 0, 0, 0, 3, 7, 8, 9), encoded)
    }

    @Test
    fun readFrameReturnsWhatWriteFrameWrote() {
        val payload = bytesOf(1, 2, 3)

        val frame = decode(encode { writeFrame(FrameType.WELCOME, payload) })

        assertEquals(FrameType.WELCOME, frame.type)
        assertArrayEquals(payload, frame.payload)
    }

    @Test
    fun readFrameAcceptsExactlyTheLimit() {
        val frame = decode(encode { writeFrame(FrameType.WELCOME, ByteArray(MAX_PAYLOAD)) })

        assertEquals(MAX_PAYLOAD, frame.payload.size)
    }

    @Test(expected = ProtocolException::class)
    fun readFrameRejectsOneByteOverTheLimit() {
        decode(bytesOf(0x10, 0x00, 0x40, 0x00, 0x01))
    }

    @Test(expected = ProtocolException::class)
    fun readFrameRejectsLengthWithTopBitSet() {
        decode(bytesOf(0x10, 0xFF, 0xFF, 0xFF, 0xFF))
    }

    @Test
    fun mediaFrameIsHeaderThenOnlyTheRequestedSlice() {
        val encoded = encode {
            writeMediaFrame(
                type = FrameType.AUDIO,
                seq = 0x01020304,
                captureTimeUs = 0x05060708090A0B0CL,
                codec = MediaHeader.CODEC_PCM_S16LE,
                data = bytesOf(9, 9, 1, 2),
                offset = 2,
                length = 2,
            )
        }

        assertArrayEquals(
            bytesOf(
                0x01, 0, 0, 0, 16, // AUDIO, length = 14-byte header + 2 bytes of data
                1, 2, 3, 4, // seq
                5, 6, 7, 8, 9, 10, 11, 12, // capture_ts
                0x01, 0, // codec, reserved
                1, 2, // data
            ),
            encoded,
        )
    }

    @Test
    fun heartbeatIsEightByteBigEndianTimestamp() {
        val payload = heartbeatPayload(0x0102030405060708L)

        assertArrayEquals(bytesOf(1, 2, 3, 4, 5, 6, 7, 8), payload)
        assertEquals(0x0102030405060708L, heartbeatSentAt(payload))
    }

    private fun encode(block: DataOutputStream.() -> Unit): ByteArray =
        ByteArrayOutputStream().also { DataOutputStream(it).block() }.toByteArray()

    private fun decode(bytes: ByteArray): Frame = DataInputStream(ByteArrayInputStream(bytes)).readFrame()

    private fun bytesOf(vararg values: Int) = ByteArray(values.size) { values[it].toByte() }
}
