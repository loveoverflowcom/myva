package com.myva.cmpprobe

import android.content.Intent
import android.os.Bundle
import android.view.ViewGroup
import androidx.activity.OnBackPressedCallback
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.platform.ViewCompositionStrategy
import com.google.androidgamesdk.GameActivity
import com.myva.probe.GameOverlay

/** B experiment: native GameActivity owns the surface; Compose owns only the HUD. */
class GameScreenActivity : GameActivity() {
    companion object {
        init { System.loadLibrary("myva_native_probe") }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        // Android may restore this Activity directly after process death, without
        // first invoking ShellActivity's launcher. Keep the process guard honest.
        ProbeProtocol.started = true
        super.onCreate(savedInstanceState)
        if (intent.getIntExtra(ProbeProtocol.VERSION_KEY, -1) != ProbeProtocol.VERSION) {
            setResult(RESULT_CANCELED)
            finish()
            return
        }
        val overlay = ComposeView(this).apply {
            setViewCompositionStrategy(ViewCompositionStrategy.DisposeOnViewTreeLifecycleDestroyed)
            setContent { GameOverlay { closeGame() } }
        }
        addContentView(overlay, ViewGroup.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT))
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() = closeGame()
        })
    }

    private fun closeGame() {
        setResult(RESULT_OK, Intent()
            .putExtra(ProbeProtocol.VERSION_KEY, ProbeProtocol.VERSION)
            .putExtra(ProbeProtocol.RESULT_KEY, "closed"))
        finish()
    }
}
