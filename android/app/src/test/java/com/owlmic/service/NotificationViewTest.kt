package com.owlmic.service

import com.owlmic.R
import com.owlmic.service.NotificationAction.CAMERA_OFF
import com.owlmic.service.NotificationAction.CAMERA_ON
import com.owlmic.service.NotificationAction.MIC_ON
import com.owlmic.service.NotificationAction.MUTE
import com.owlmic.service.NotificationAction.STOP
import com.owlmic.service.NotificationAction.UNMUTE
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NotificationViewTest {
    private val usb = Link.Live(1, setOf("vcam"))
    private val micLive = OwlmicState(micOn = true, link = usb)

    @Test
    fun micLive() {
        val view = notificationView(micLive)
        assertEquals(R.string.notification_live, view.title)
        assertEquals(listOf(R.string.notification_mic, R.string.notification_link_usb), view.parts)
        assertEquals(NotificationIcon.MIC, view.icon)
        assertTrue(view.live)
        assertEquals(listOf(MUTE, CAMERA_ON, STOP), view.actions)
    }

    @Test
    fun micAndCameraLive() {
        val view = notificationView(micLive.copy(camera = CameraState(on = true)))
        assertEquals(listOf(R.string.notification_mic, R.string.notification_camera, R.string.notification_link_usb), view.parts)
        assertEquals(listOf(MUTE, CAMERA_OFF, STOP), view.actions)
    }

    @Test
    fun cameraOnlyLiveOffersTheMicBack() {
        val view = notificationView(OwlmicState(link = usb, camera = CameraState(on = true)))
        assertEquals(NotificationIcon.CAMERA, view.icon)
        assertTrue(view.live)
        assertEquals(listOf(MIC_ON, CAMERA_OFF, STOP), view.actions)
    }

    @Test
    fun mutedWithCameraOffIsNotLive() {
        val view = notificationView(micLive.copy(muted = true, link = Link.Live(3, setOf("vcam"))))
        assertEquals(R.string.notification_muted, view.title)
        assertEquals(listOf(R.string.notification_camera_off, R.string.notification_link_wifi), view.parts)
        assertEquals(NotificationIcon.MIC_OFF, view.icon)
        assertFalse(view.live)
        assertEquals(listOf(UNMUTE, CAMERA_ON, STOP), view.actions)
    }

    @Test
    fun mutedWithCameraLiveStaysLive() {
        val view = notificationView(micLive.copy(muted = true, camera = CameraState(on = true)))
        assertTrue(view.live)
        assertEquals(NotificationIcon.CAMERA, view.icon)
        assertEquals(listOf(R.string.notification_mic_muted, R.string.notification_camera, R.string.notification_link_usb), view.parts)
    }

    @Test
    fun noCameraButtonWhereVideoCantGo() {
        // Bluetooth can't carry video, and a PC without a virtual camera can't show it.
        assertEquals(listOf(MUTE, STOP), notificationView(micLive.copy(link = Link.Live(4, setOf("vcam")))).actions)
        assertEquals(listOf(MUTE, STOP), notificationView(micLive.copy(link = Link.Live(1))).actions)
    }

    @Test
    fun cameraOnButBlockedIsNotLive() {
        val view = notificationView(OwlmicState(link = Link.Live(4), camera = CameraState(on = true)))
        assertEquals(R.string.notification_connected, view.title)
        assertFalse(view.live)
        assertEquals(listOf(MIC_ON, CAMERA_OFF, STOP), view.actions)
    }

    @Test
    fun withoutAPcOnlyStop() {
        val looking = notificationView(OwlmicState(micOn = true))
        assertEquals(R.string.notification_looking, looking.title)
        assertEquals(listOf(R.string.notification_mic_on, R.string.notification_camera_off), looking.parts)
        assertFalse(looking.live)
        assertEquals(listOf(STOP), looking.actions)

        assertEquals(R.string.notification_reconnecting, notificationView(OwlmicState(micOn = true, reconnecting = true)).title)
        assertEquals(R.string.notification_waiting, notificationView(OwlmicState(micOn = true, link = Link.Waiting)).title)
    }

    @Test
    fun refusedSaysWhatToDo() {
        val denied = notificationView(OwlmicState(micOn = true, link = Link.Refused("denied")))
        assertEquals(R.string.notification_denied, denied.title)
        assertEquals(listOf(R.string.notification_try_again), denied.parts)
        assertEquals(listOf(STOP), denied.actions)
        assertEquals(R.string.notification_update, notificationView(OwlmicState(micOn = true, link = Link.Refused("version"))).title)
        assertEquals(R.string.notification_refused, notificationView(OwlmicState(micOn = true, link = Link.Refused("busy"))).title)
    }
}
