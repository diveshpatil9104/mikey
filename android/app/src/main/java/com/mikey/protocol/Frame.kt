package com.mikey.protocol

import java.io.DataInputStream
import java.io.DataOutputStream
import java.net.ProtocolException

/** Largest payload either side accepts. Anything bigger closes the connection. */
const val MAX_PAYLOAD = 4 * 1024 * 1024

/** One frame on the wire: type u8 | payload length u32 BE | payload. */
class Frame(val type: Int, val payload: ByteArray)

fun DataOutputStream.writeFrame(type: Int, payload: ByteArray) {
    writeByte(type)
    writeInt(payload.size)
    write(payload)
}

/** Writes an AUDIO or VIDEO frame straight from [data], without building a payload array first. */
fun DataOutputStream.writeMediaFrame(
    type: Int,
    seq: Int,
    captureTimeUs: Long,
    codec: Int,
    data: ByteArray,
    offset: Int,
    length: Int,
) {
    writeByte(type)
    writeInt(MediaHeader.SIZE + length)
    MediaHeader.write(this, seq, captureTimeUs, codec)
    write(data, offset, length)
}

fun DataInputStream.readFrame(): Frame {
    val type = readUnsignedByte()
    // The length is a u32; with the top bit set it reads negative here, and is over the limit anyway.
    val length = readInt()
    if (length !in 0..MAX_PAYLOAD) throw ProtocolException("Frame too large: ${length.toUInt()} bytes")
    val payload = ByteArray(length)
    readFully(payload)
    return Frame(type, payload)
}
