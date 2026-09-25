package com.mikey.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.mikey.R

/** Green when the stream reaches the PC, red when there is no PC. */
@Composable
fun StatusDot(connected: Boolean, modifier: Modifier = Modifier) {
    val description = stringResource(if (connected) R.string.cd_status_connected else R.string.cd_status_no_pc)
    Box(
        modifier
            .size(10.dp)
            .background(if (connected) Palette.statusOk else Palette.statusErr, RoundedCornerShape(3.dp))
            .semantics { contentDescription = description },
    )
}
