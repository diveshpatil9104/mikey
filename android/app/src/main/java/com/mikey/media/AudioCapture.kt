package com.mikey.media

import android.annotation.SuppressLint
import android.content.Context
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioRecord
import android.media.MediaRecorder
import android.os.Process
import android.os.SystemClock
import android.util.Log

/** 10 ms of 48 kHz mono PCM s16le, as recorded. */
class AudioFrame(val seq: Int, val captureTimeUs: Long, val pcm: ByteArray)

/**
 * Records raw audio (48 kHz, mono, 16-bit) in 10 ms frames on its own high-priority thread.
 * No processing on the phone: noise suppression and echo cancellation run on the PC.
 * [onFrame] runs on the capture thread and must never block.
 */
class AudioCapture(private val context: Context, private val onFrame: (AudioFrame) -> Unit) {
    @Volatile private var running = false

    fun start() {
        running = true
        Thread(::record, "mikey-capture").start()
    }

    /** Returns at once. The mic is released within one frame. */
    fun stop() {
        running = false
    }

    // MikeyService only runs after RECORD_AUDIO is granted.
    @SuppressLint("MissingPermission")
    private fun record() {
        Process.setThreadPriority(Process.THREAD_PRIORITY_URGENT_AUDIO)
        val bufferBytes = maxOf(AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL, ENCODING), FRAME_BYTES * 2)
        val recorder = try {
            AudioRecord(source(), SAMPLE_RATE, CHANNEL, ENCODING, bufferBytes)
        } catch (e: IllegalArgumentException) {
            Log.e(TAG, "Can't create the recorder", e)
            return
        }
        try {
            recorder.startRecording()
            var seq = 0
            while (running) {
                val pcm = ByteArray(FRAME_BYTES)
                val read = recorder.read(pcm, 0, FRAME_BYTES)
                if (read < 0) {
                    Log.e(TAG, "Recording stopped with error $read")
                    break
                }
                if (read == FRAME_BYTES) onFrame(AudioFrame(seq++, SystemClock.elapsedRealtimeNanos() / 1000, pcm))
            }
        } catch (e: IllegalStateException) {
            // For example another app holds the mic.
            Log.e(TAG, "Can't record", e)
        } finally {
            recorder.release()
        }
    }

    private fun source(): Int {
        val unprocessed = context.getSystemService(AudioManager::class.java)
            .getProperty(AudioManager.PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED) == "true"
        // Never VOICE_COMMUNICATION: its noise suppression and gain would hurt the PC's echo canceller.
        return if (unprocessed) MediaRecorder.AudioSource.UNPROCESSED else MediaRecorder.AudioSource.MIC
    }

    private companion object {
        const val TAG = "AudioCapture"
        const val SAMPLE_RATE = 48_000
        const val CHANNEL = AudioFormat.CHANNEL_IN_MONO
        const val ENCODING = AudioFormat.ENCODING_PCM_16BIT
        const val FRAME_BYTES = SAMPLE_RATE / 100 * 2
    }
}
