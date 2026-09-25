package com.mikey

import android.Manifest
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts.RequestMultiplePermissions
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import com.mikey.service.MikeyService
import com.mikey.ui.SplitScreen

class MainActivity : ComponentActivity() {

    private val requestMicPermission = registerForActivityResult(RequestMultiplePermissions()) { granted ->
        if (granted[Manifest.permission.RECORD_AUDIO] == true) MikeyService.micOn(this)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge(
            statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
            navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
        )
        setContent {
            val state by MikeyService.state.collectAsState()
            SplitScreen(state, onMicTap = ::onMicTap)
        }
    }

    private fun onMicTap() {
        when {
            MikeyService.state.value.micOn -> MikeyService.micOff(this)
            checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED ->
                MikeyService.micOn(this)
            else -> requestMicPermission.launch(micPermissions())
        }
    }
}

/** Asked only on the first mic tap. Android 13+ also needs notification permission for the status notification. */
private fun micPermissions(): Array<String> =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        arrayOf(Manifest.permission.RECORD_AUDIO, Manifest.permission.POST_NOTIFICATIONS)
    } else {
        arrayOf(Manifest.permission.RECORD_AUDIO)
    }
