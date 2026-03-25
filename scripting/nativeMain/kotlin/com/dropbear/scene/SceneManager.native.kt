@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.scene

import com.dropbear.DropbearEngine
import com.dropbear.ffi.generated.*
import kotlin.String
import kotlinx.cinterop.*

internal actual fun SceneManager.loadSceneAsyncNative(sceneName: String): SceneLoadHandle? = memScoped {
    val cmd = DropbearEngine.native.commandBufferHandle ?: return@memScoped null
    val sceneLoader = DropbearEngine.native.sceneLoaderHandle ?: return@memScoped null
    val out = alloc<ULongVar>()
    val rc = dropbear_scripting_load_scene_async(cmd, sceneLoader, sceneName, out.ptr)
    if (rc != 0) null else SceneLoadHandle(out.value.toLong())
}

internal actual fun SceneManager.loadSceneAsyncNative(sceneName: String, loadingScene: String): SceneLoadHandle? = memScoped {
    val cmd = DropbearEngine.native.commandBufferHandle ?: return@memScoped null
    val sceneLoader = DropbearEngine.native.sceneLoaderHandle ?: return@memScoped null
    val out = alloc<ULongVar>()
    val rc = dropbear_scripting_load_scene_async_with_loading(cmd, sceneLoader, sceneName, loadingScene, out.ptr)
    if (rc != 0) null else SceneLoadHandle(out.value.toLong())
}

internal actual fun SceneManager.switchToSceneImmediateNative(sceneName: String) {
    val cmd = DropbearEngine.native.commandBufferHandle ?: return
    memScoped { dropbear_scripting_switch_to_scene_immediate(cmd, sceneName) }
}