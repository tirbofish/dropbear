package com.dropbear.scene

import com.dropbear.utils.Progress
import com.dropbear.exception.PrematureSceneSwitchException
import com.dropbear.exceptionOnError
import com.dropbear.ffi.NativeEngine

/**
 * A handle that allows you to check the state of an async scene load.
 */
class SceneLoadHandle(val id: Long) {
    val sceneName: String
        get() = getSceneLoadHandleSceneName(id)
    /**
     * Switches the scene to the requested scene.
     *
     * This function assumes that you have checked its progress and has checked if
     * it has succeeded or failed, and whether it is ready to be switched.
     *
     * If not checked, it will throw a [PrematureSceneSwitchException], even if [exceptionOnError]
     * is enabled or not.
     */
    fun switchTo() {
        switchToSceneAsync()
    }

    /**
     * Returns the progress of scene load.
     */
    fun progress(): Progress {
        return getSceneLoadProgress()
    }

    fun status(): SceneLoadStatus {
        return getSceneLoadStatus()
    }

    /**
     * Checks if the scene load has completed.
     *
     * If completed, it will return true. If not, it will return false.
     *
     * If completed, it is recommended that you queue up the switch with [switchTo].
     */
    fun isComplete(): Boolean {
        return status() == SceneLoadStatus.READY
    }

    /**
     * Checks if the scene load has failed.
     *
     * If failed, it will return true. If not, it will return false.
     *
     * If failed, it is recommended that you handle the error with [error].
     */
    fun hasFailed(): Boolean {
        return status() == SceneLoadStatus.FAILED
    }

    /**
     * Returns the raw id of the handle.
     */
    fun raw(): Long {
        return id
    }
}

expect fun SceneLoadHandle.getSceneLoadHandleSceneName(id: Long): String
expect fun SceneLoadHandle.switchToSceneAsync()
expect fun SceneLoadHandle.getSceneLoadProgress(): Progress
expect fun SceneLoadHandle.getSceneLoadStatus(): SceneLoadStatus