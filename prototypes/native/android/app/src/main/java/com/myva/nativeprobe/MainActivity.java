package com.myva.nativeprobe;

import com.google.androidgamesdk.GameActivity;

/** Standalone baseline. This Activity contains no Compose UI or CMP integration. */
public final class MainActivity extends GameActivity {
    static {
        System.loadLibrary("myva_native_probe");
    }
}
