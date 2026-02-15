package com.dropbear.ffi

import com.dropbear.logging.Logger

actual class NativeEngine {
    internal var worldHandle: Long = 0L
    internal var inputHandle: Long = 0L
    internal var commandBufferHandle: Long = 0L
    internal var graphicsContextHandle: Long = 0L
    internal var assetHandle: Long = 0L
    internal var sceneLoaderHandle: Long = 0L
    internal var physicsEngineHandle: Long = 0L
    internal var uiBufferHandle: Long = 0L

    @JvmName("init")
    fun init(ctx: DropbearContext) {
        this.worldHandle = ctx.worldHandle
        this.inputHandle = ctx.inputHandle
        this.commandBufferHandle = ctx.commandBufferHandle
        this.graphicsContextHandle = ctx.graphicsContextHandle
        this.assetHandle = ctx.assetHandle
        this.sceneLoaderHandle = ctx.sceneLoaderHandle
        this.physicsEngineHandle = ctx.physicsEngineHandle
        this.uiBufferHandle = ctx.uiHandle

        if (this.worldHandle <= 0L) {
            Logger.error("NativeEngine: Error - Invalid world handle received!")
            return
        }
        if (this.inputHandle <= 0L) {
            Logger.error("NativeEngine: Error - Invalid input handle received!")
            return
        }
        if (this.commandBufferHandle <= 0L) {
            Logger.error("NativeEngine: Error - Invalid graphics handle received!")
            return
        }
        if (this.graphicsContextHandle <= 0L) {
            Logger.error("NativeEngine: Error - Invalid graphics context handle received!")
            return
        }
        if (this.assetHandle <= 0L) {
            Logger.error("NativeEngine: Error - Invalid asset handle received!")
            return
        }
        if (this.sceneLoaderHandle <= 0L) {
            Logger.error("NativeEngine: Error - Invalid scene loader handle received!")
            return
        }
        if (this.physicsEngineHandle <= 0L) {
            Logger.error("NativeEngine: Error - Invalid physics handle received!")
            return
        }
        if (this.uiBufferHandle <= 0L) {
            Logger.error("NativeEngine: Error - Invalid ui command buffer handle received!")
            return
        }
    }
}