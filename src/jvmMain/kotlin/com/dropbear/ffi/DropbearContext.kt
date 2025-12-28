package com.dropbear.ffi

/**
 * Contains all the pointers available.
 */
class DropbearContext(
    val worldHandle: Long,
    val inputHandle: Long,
    val commandBufferHandle: Long,
    val assetHandle: Long,
    val sceneLoaderHandle: Long,
    val physicsEngineHandle: Long,
)