package com.dropbear.scene

import com.dropbear.DropbearEngine
import com.dropbear.utils.Progress

internal actual fun SceneLoadHandle.getSceneLoadHandleSceneName(id: Long): String {
    return SceneLoadHandleNative.getSceneLoadHandleSceneName(DropbearEngine.native.sceneLoaderHandle, id)
}

internal actual fun SceneLoadHandle.switchToSceneAsync() {
    return SceneLoadHandleNative.switchToSceneAsync(DropbearEngine.native.commandBufferHandle, this.id)
}

internal actual fun SceneLoadHandle.getSceneLoadProgress(): Progress {
    return SceneLoadHandleNative.getSceneLoadProgress(DropbearEngine.native.sceneLoaderHandle, this.id)
}

internal actual fun SceneLoadHandle.getSceneLoadStatus(): SceneLoadStatus {
    val result = SceneLoadHandleNative.getSceneLoadStatus(DropbearEngine.native.sceneLoaderHandle, this.id)
    return SceneLoadStatus.entries[result]
}