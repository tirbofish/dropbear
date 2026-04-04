rootProject.name = "surface-nets-plugin"

pluginManagement {
    repositories {
        mavenLocal()
        gradlePluginPortal()
        maven { url = uri("https://tirbofish.github.io/dropbear/") }
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.8.0"
}

buildCache {
    local {
        isEnabled = true
    }
}
