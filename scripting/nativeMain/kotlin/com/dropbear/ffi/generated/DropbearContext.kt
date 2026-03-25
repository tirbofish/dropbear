@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.dropbear.ffi.generated

import kotlinx.cinterop.NativePtr

data class NativeHandle(
    val rawValue: NativePtr?
)

data class DropbearContext(
    val world: NativeHandle?,
    val input: NativeHandle?,
    val commandBuffer: NativeHandle?,
    val graphicsContext: NativeHandle?,
    val assets: NativeHandle?,
    val sceneLoader: NativeHandle?,
    val physicsEngine: NativeHandle?,
    val uiBuffer: NativeHandle?
)
