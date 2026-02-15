package com.dropbear.scene

import com.dropbear.DropbearEngine

internal actual fun SceneManager.loadSceneAsyncNative(sceneName: String): SceneLoadHandle? {
    val result = SceneManagerNative.loadSceneAsync(
        DropbearEngine.native.commandBufferHandle,
        DropbearEngine.native.sceneLoaderHandle,
        sceneName
    )
    return SceneLoadHandle(result)
}

internal actual fun SceneManager.loadSceneAsyncNative(
    sceneName: String,
    loadingScene: String
): SceneLoadHandle? {
    val result = SceneManagerNative.loadSceneAsyncWithLoading(
        DropbearEngine.native.commandBufferHandle,
        DropbearEngine.native.sceneLoaderHandle,
        sceneName,
        loadingScene
    )
    return SceneLoadHandle(result)
}

internal actual fun SceneManager.switchToSceneImmediateNative(sceneName: String) {
    SceneManagerNative.switchToSceneImmediate(
        DropbearEngine.native.commandBufferHandle,
        sceneName
    )
}