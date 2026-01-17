package com.dropbear.scene

/**
 * A manager for dealing with scene loading and scene querying. 
 */
class SceneManager() {
    /**
     * Loads the resources of a scene asynchronously.
     *
     * dropbear loads its resources on another thread, which means your window is still responsive.
     * After running this function, it is recommended that you check up/poll the [SceneLoadHandle]
     * and check its status.
     */
    fun loadSceneAsync(sceneName: String): SceneLoadHandle? {
        return loadSceneAsyncNative(sceneName)
    }

    /**
     * Loads the resources of a scene asynchronously.
     *
     * This function is an overload, which contains a `loadingScene` parameter.
     * This allows you to specify a scene to display while the resources are being loaded.
     *
     * It must be preloaded (through the scene settings menu in eucalyptus-editor). If not preloaded, it will
     * block/freeze the main thread/window to load the `loadingScene`
     */
    fun loadSceneAsync(sceneName: String, loadingScene: String): SceneLoadHandle? {
        return loadSceneAsyncNative(sceneName, loadingScene)
    }

    /**
     * Switches the scene on the next frame. This is an immediate function, which
     * means it will block/freeze the window until all resources are loaded.
     *
     * Using this is not recommended unless your scene contains a few assets, such
     * as 3D objects. Use [loadSceneAsync] instead.
     */
    fun switchToSceneImmediate(sceneName: String) {
        return switchToSceneImmediateNative(sceneName)
    }
}

internal expect fun SceneManager.loadSceneAsyncNative(sceneName: String): SceneLoadHandle?
internal expect fun SceneManager.loadSceneAsyncNative(sceneName: String, loadingScene: String): SceneLoadHandle?
internal expect fun SceneManager.switchToSceneImmediateNative(sceneName: String)