@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.scene

import com.dropbear.DropbearEngine
import com.dropbear.ffi.generated.*
import kotlin.String
import com.dropbear.utils.Progress
import kotlinx.cinterop.*

internal actual fun SceneLoadHandle.switchToSceneAsync() {
    val cmd = DropbearEngine.native.commandBufferHandle ?: return
    val sceneLoader = DropbearEngine.native.sceneLoaderHandle ?: return
    memScoped { dropbear_scripting_switch_to_scene_async(cmd, sceneLoader, id.toULong()) }
}

internal actual fun SceneLoadHandle.getSceneLoadProgress(): Progress = memScoped {
    val sceneLoader = DropbearEngine.native.sceneLoaderHandle ?: return@memScoped Progress.nothing()
    val out = alloc<com.dropbear.ffi.generated.Progress>()
    val rc = dropbear_scripting_get_scene_load_progress(sceneLoader, id.toULong(), out.ptr)
    if (rc != 0) Progress.nothing() else Progress(
        out.current.toDouble(),
        out.total.toDouble(),
        out.message?.toKString()
    )
}

internal actual fun SceneLoadHandle.getSceneLoadStatus(): SceneLoadStatus = memScoped {
    val sceneLoader = DropbearEngine.native.sceneLoaderHandle ?: return@memScoped SceneLoadStatus.FAILED
    val out = alloc<UIntVar>()
    val rc = dropbear_scripting_get_scene_load_status(sceneLoader, id.toULong(), out.ptr)
    if (rc != 0) SceneLoadStatus.FAILED else when (out.value.toInt()) {
        0 -> SceneLoadStatus.PENDING
        1 -> SceneLoadStatus.READY
        else -> SceneLoadStatus.FAILED
    }
}

internal actual fun SceneLoadHandle.getSceneLoadHandleSceneName(id: Long): String = memScoped {
    val sceneLoader = DropbearEngine.native.sceneLoaderHandle ?: return@memScoped ""
    val out = alloc<CPointerVar<ByteVar>>()
    val rc = dropbear_scripting_get_scene_load_handle_scene_name(sceneLoader, id.toULong(), out.ptr)
    if (rc != 0) "" else out.value?.toKString() ?: ""
}