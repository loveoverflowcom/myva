plugins {
    id("org.jetbrains.kotlin.multiplatform")
    id("com.android.kotlin.multiplatform.library")
    id("org.jetbrains.kotlin.plugin.compose")
    id("org.jetbrains.compose")
}
kotlin {
    android {
        namespace = "com.myva.probe.shared"
        compileSdk = 37
        minSdk = 26
    }
    sourceSets.commonMain.dependencies {
        implementation("org.jetbrains.compose.runtime:runtime:1.12.0")
        implementation("org.jetbrains.compose.foundation:foundation:1.12.0")
        implementation("org.jetbrains.compose.ui:ui:1.12.0")
    }
}
