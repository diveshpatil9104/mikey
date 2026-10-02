package com.owlmic.ui

import com.owlmic.R
import com.owlmic.media.Lens
import com.owlmic.service.CameraBlock
import com.owlmic.service.CameraState
import com.owlmic.service.Link
import com.owlmic.service.OwlmicState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ScreenModelTest {
    private val usb = Link.Live(1, setOf("pcm", "opus", "vcam", "rnnoise"))

    @Test
    fun theMicIsInvitingWhenIdleAndInvertedOnlyWhenThePcReceives() {
        assertEquals(Look.READY, micView(OwlmicState(), denied = false).look)
        assertEquals(Look.LIVE, micView(OwlmicState(micOn = true, link = usb), denied = false).look)
        assertEquals(Look.MUTED, micView(OwlmicState(micOn = true, link = usb, muted = true), denied = false).look)
        assertEquals(Look.DENIED, micView(OwlmicState(), denied = true).look)
    }

    @Test
    fun aMicThatIsOnButNotReachingThePcSaysWhy() {
        val first = micView(OwlmicState(micOn = true), denied = false)
        assertEquals(listOf(R.string.label_mic_on, R.string.label_no_pc), first.label)
        assertFalse(first.lineStrong)

        val dropped = micView(OwlmicState(micOn = true, reconnecting = true), denied = false)
        assertEquals(R.string.line_dropped, dropped.line)
        assertTrue(dropped.lineStrong)

        assertEquals(R.string.line_waiting, micView(OwlmicState(micOn = true, link = Link.Waiting), denied = false).line)
        assertEquals(R.string.line_refused_version, micView(OwlmicState(micOn = true, link = Link.Refused("version")), denied = false).line)
    }

    @Test
    fun theCameraNamesItsLensAndIsUnavailableWhereVideoCantGo() {
        val back = cameraView(OwlmicState(camera = CameraState(on = true), link = usb))
        assertEquals(Look.LIVE, back.look)
        assertEquals(listOf(R.string.camera_back), back.label)

        val front = cameraView(OwlmicState(camera = CameraState(on = true, lens = Lens.FRONT)))
        assertEquals(listOf(R.string.camera_front, R.string.label_no_pc), front.label)

        assertEquals(CameraBlock.BLUETOOTH, cameraBlock(OwlmicState(link = Link.Live(4))))
        assertEquals(CameraBlock.PC, cameraBlock(OwlmicState(link = Link.Live(3, setOf("pcm", "opus")))))
        assertNull(cameraBlock(OwlmicState(link = usb)))
        assertEquals(Look.BLOCKED, cameraView(OwlmicState(link = Link.Live(4))).look)
    }

    @Test
    fun theOtherHalfIsDimWhileTheLinkIsDown() {
        assertEquals(Look.DIM, micView(OwlmicState(camera = CameraState(on = true)), denied = false).look)
        assertEquals(Look.READY, micView(OwlmicState(camera = CameraState(on = true), link = usb), denied = false).look)
    }

    @Test
    fun settingsTheConnectedPcCantDoAreGreyedOut() {
        assertTrue(pcCan(OwlmicState(), "rnnoise"))
        assertTrue(pcCan(OwlmicState(link = usb), "rnnoise"))
        assertFalse(pcCan(OwlmicState(link = Link.Live(1, setOf("pcm"))), "rnnoise"))
    }

    @Test
    fun theSheetHeaderFollowsTheLink() {
        assertEquals(Header(R.string.sheet_connected, R.string.level_usb_debugging), sheetHeader(OwlmicState(micOn = true, link = usb)))
        assertEquals(Header(R.string.sheet_idle), sheetHeader(OwlmicState()))
        assertEquals(Header(R.string.sheet_reconnecting), sheetHeader(OwlmicState(micOn = true, reconnecting = true)))
        assertEquals(R.string.level_wifi, levelName(3))
    }

    @Test
    fun theRingLightsFromTheBottomWithTheVoice() {
        assertEquals(0, litDots(0f))
        assertEquals(12, litDots(0.5f))
        assertEquals(25, litDots(1f)) // All 24 on each side, top included.
    }
}
