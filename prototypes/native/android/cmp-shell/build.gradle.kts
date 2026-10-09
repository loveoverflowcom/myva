plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
    id("org.jetbrains.compose")
}
android {
    namespace = "com.myva.cmpprobe"
    compileSdk = 37
    buildToolsVersion = "36.0.0"
    ndkVersion = "27.0.12077973"
    defaultConfig {
        applicationId = "com.myva.cmpprobe"
        minSdk = 26
        targetSdk = 37
        versionCode = 1
        versionName = "0.1.0"
        ndk { abiFilters += "arm64-v8a" }
    }
    buildFeatures { compose = true }
    sourceSets.getByName("main").jniLibs.srcDir("../app/src/main/jniLibs")
}
dependencies {
    implementation(project(":shared"))
    implementation("org.jetbrains.compose.runtime:runtime:1.12.0")
    implementation("org.jetbrains.compose.ui:ui:1.12.0")
    implementation("androidx.activity:activity-compose:1.12.4")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.games:games-activity:4.4.0")
}
