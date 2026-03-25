@file:OptIn(ExperimentalForeignApi::class, ExperimentalNativeApi::class)
@file:Suppress("EXPECT_ACTUAL_CLASSIFIERS_ARE_IN_BETA_WARNING")

package com.dropbear.ffi

import com.dropbear.exception.DropbearNativeException
import com.dropbear.exceptionOnError
import com.dropbear.ffi.generated.DropbearContext
import com.dropbear.logging.Logger
import kotlinx.cinterop.COpaquePointer
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.interpretCPointer
import kotlin.experimental.ExperimentalNativeApi

actual class NativeEngine {
    internal var worldHandle: COpaquePointer? = null
    internal var inputHandle: COpaquePointer? = null
    internal var commandBufferHandle: COpaquePointer? = null
    internal var graphicsContextHandle: COpaquePointer? = null
    internal var assetHandle: COpaquePointer? = null
    internal var sceneLoaderHandle: COpaquePointer? = null
    internal var physicsEngineHandle: COpaquePointer? = null
    internal var uiBufferHandle: COpaquePointer? = null

    @Suppress("unused")
    fun init(
        ctx: DropbearContext?
    ) {
        this.worldHandle = ctx?.world?.rawValue?.let { interpretCPointer(it) }
        this.inputHandle = ctx?.input?.rawValue?.let { interpretCPointer(it) }
        this.commandBufferHandle = ctx?.commandBuffer?.rawValue?.let { interpretCPointer(it) }
        this.graphicsContextHandle = ctx?.graphicsContext?.rawValue?.let { interpretCPointer(it) }
        this.assetHandle = ctx?.assets?.rawValue?.let { interpretCPointer(it) }
        this.sceneLoaderHandle = ctx?.sceneLoader?.rawValue?.let { interpretCPointer(it) }
        this.physicsEngineHandle = ctx?.physicsEngine?.rawValue?.let { interpretCPointer(it) }
        this.uiBufferHandle = ctx?.uiBuffer?.rawValue?.let { interpretCPointer(it) }

        Logger.init(com.dropbear.logging.SocketWriter())

        // if release, always enable exceptionOnError
        if (!Platform.isDebugBinary) {
            exceptionOnError = true
        }

        if (this.worldHandle == null) {
            Logger.error("NativeEngine: Error - Invalid world handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid world handle received!")
            }
        }
        if (this.inputHandle == null) {
            Logger.error("NativeEngine: Error - Invalid input handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid input handle received!")
            }
        }
        if (this.commandBufferHandle == null) {
            Logger.error("NativeEngine: Error - Invalid graphics handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid graphics handle received!")
            }
        }
        if (this.assetHandle == null) {
            Logger.error("NativeEngine: Error - Invalid asset handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid asset handle received!")
            }
        }
        if (this.physicsEngineHandle == null) {
            Logger.error("NativeEngine: Error - Invalid physics engine handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid physics engine handle received!")
            }
        }
        if (this.uiBufferHandle == null) {
            Logger.error("NativeEngine: Error - Invalid ui command buffer engine handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid ui command buffer engine handle received!")
            }
        }
    }
}