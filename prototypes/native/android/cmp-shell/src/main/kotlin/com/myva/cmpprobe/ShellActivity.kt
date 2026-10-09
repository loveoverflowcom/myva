package com.myva.cmpprobe

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.myva.probe.ProbeShell

internal object ProbeProtocol {
    const val VERSION = 1
    const val VERSION_KEY = "myva.protocol_version"
    const val RESULT_KEY = "myva.result"
    // Process scope is intentional: winit cannot recreate an event loop on return.
    var started = false
}

class ShellActivity : ComponentActivity() {
    private var available by mutableStateOf(!ProbeProtocol.started)
    private var status by mutableStateOf("Bản thử hiện chỉ mở cảnh một lần mỗi phiên ứng dụng.")
    private val game = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        available = false
        val version = result.data?.getIntExtra(ProbeProtocol.VERSION_KEY, -1)
        val resultName = result.data?.getStringExtra(ProbeProtocol.RESULT_KEY)
        status = if (result.resultCode == RESULT_OK && version == ProbeProtocol.VERSION && resultName == "closed") {
            "Đã đóng cảnh. Để thử lại, hãy dừng ứng dụng và mở lại."
        } else {
            "Cảnh đã dừng hoặc trả kết quả không tương thích. Hãy xem log của bản thử."
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            ProbeShell(available, status) {
                if (!ProbeProtocol.started) {
                    ProbeProtocol.started = true
                    available = false
                    game.launch(Intent(this, GameScreenActivity::class.java)
                        .putExtra(ProbeProtocol.VERSION_KEY, ProbeProtocol.VERSION))
                }
            }
        }
    }
}
