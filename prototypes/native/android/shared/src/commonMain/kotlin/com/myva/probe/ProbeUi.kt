package com.myva.probe

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private val Ink = Color(0xFFF0F6FF)
private val Panel = Color(0xFF152D40)

@Composable
fun ProbeShell(available: Boolean, status: String, onOpen: () -> Unit) {
    Column(
        Modifier.fillMaxSize().background(Color(0xFF071322)).safeDrawingPadding().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(20.dp),
    ) {
        BasicText("MyVa · Thử nghiệm native", style = TextStyle(color = Ink, fontSize = 26.sp))
        BasicText("Cảnh di chuyển bằng touch, với thanh điều khiển Compose.", style = TextStyle(color = Ink))
        Action("Mở cảnh thử", available, onOpen)
        BasicText(status, style = TextStyle(color = Ink))
    }
}

@Composable
fun GameOverlay(onClose: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().background(Panel).safeDrawingPadding().padding(12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        BasicText("MyVa · Kéo để di chuyển", style = TextStyle(color = Ink))
        Action("Đóng", true, onClose)
    }
}

@Composable
private fun Action(label: String, enabled: Boolean, onClick: () -> Unit) {
    BasicText(
        label,
        modifier = Modifier.background(if (enabled) Color(0xFF087E71) else Panel)
            .clickable(enabled = enabled, onClick = onClick).padding(14.dp),
        style = TextStyle(color = Ink, fontSize = 16.sp),
    )
}
