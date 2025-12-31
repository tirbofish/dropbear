package com.dropbear.scene

import com.dropbear.DropbearEngine

actual fun SceneManager.loadSceneAsyncNative(sceneName: String): SceneLoadHandle? {
    val result = SceneManagerNative.loadSceneAsyncNative(
        DropbearEngine.native.commandBufferHandle,
        DropbearEngine.native.sceneLoaderHandle,
        sceneName
    )
    return if (result != null) SceneLoadHandle(result) else null
}

actual fun SceneManager.loadSceneAsyncNative(
    sceneName: String,
    loadingScene: String
): SceneLoadHandle? {
    val result = SceneManagerNative.loadSceneAsyncNative(
        DropbearEngine.native.commandBufferHandle,
        DropbearEngine.native.sceneLoaderHandle,
        sceneName,
        loadingScene
    )
    return if (result != null) SceneLoadHandle(result) else null
}

actual fun SceneManager.switchToSceneImmediateNative(sceneName: String) {
    SceneManagerNative.switchToSceneImmediateNative(
        DropbearEngine.native.commandBufferHandle,
        sceneName
    )
}