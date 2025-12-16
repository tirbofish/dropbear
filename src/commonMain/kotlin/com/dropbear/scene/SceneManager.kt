package com.dropbear.scene

import com.dropbear.ffi.NativeEngine

class SceneManager(val native: NativeEngine) {
    /**
     * Loads the resources of a scene asynchronously.
     *
     * dropbear loads its resources on another thread, which means your window is still responsive.
     * After running this function, it is recommended that you check up/poll the [SceneLoadHandle]
     * and check its status.
     */
    fun loadSceneAsync(sceneName: String): SceneLoadHandle? {
        return native.loadSceneAsync(sceneName)
    }

    /**
     * Loads the resources of a scene asynchronously.
     *
     * This function is an overload, which contains a `loading_scene` parameter.
     * This allows you to specify a scene to display while the resources are being loaded.
     *
     * It must be preloaded (through the scene settings menu in eucalyptus-editor). If not preloaded, it will
     * block/freeze the main thread/window to load the `loading_scene`
     */
    fun loadSceneAsync(sceneName: String, loading_scene: String): SceneLoadHandle? {
        return native.loadSceneAsync(sceneName, loading_scene)
    }

    /**
     * Switches the scene on the next frame. This is an immediate function, which
     * means it will block/freeze the window until all resources are loaded.
     *
     * Using this is not recommended unless your scene contains a few assets, such
     * as 3D objects. Use [loadSceneAsync] instead.
     */
    fun switchToSceneImmediate(sceneName: String) {
        return native.switchToSceneImmediate(sceneName)
    }
}