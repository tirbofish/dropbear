@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.dropbear.ffi.generated

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.NativePtr

data class NativeHandle(
    val rawValue: NativePtr?
)

data class DropbearContext(
    val world: NativeHandle?,
    val input: NativeHandle?,
    val graphics: NativeHandle?,
    val assets: NativeHandle?,
    val scene_loader: NativeHandle?,
    val physics_engine: NativeHandle?
)
