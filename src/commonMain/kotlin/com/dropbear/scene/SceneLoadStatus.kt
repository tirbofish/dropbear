package com.dropbear.scene

/**
 * Determines the current loading status of a scene, as queried from
 * [SceneLoadHandle.status].
 */
enum class SceneLoadStatus {
    /**
     * The scene is currently being loaded is not ready. It has not failed yet either.
     */
    PENDING,

    /**
     * The scene has all of its objects loaded into memory and is ready to be switched.
     */
    READY,

    /**
     * The scene has failed during the process of loading, and cannot be recovered.
     *
     * Either send another request through [SceneManager.loadSceneAsync] and set the handle to
     * `null`.
     */
    FAILED
}