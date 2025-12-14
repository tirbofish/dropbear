package com.dropbear.scene

import com.dropbear.DropbearEngine
import com.dropbear.utils.Progress
import com.dropbear.exception.PrematureSceneSwitchException
import com.dropbear.exceptionOnError

/**
 * A handle that allows you to check the state of an async scene load.
 */
class SceneLoadHandle(val id: Long, val sceneName: String, var result: SceneLoadStatus) {
    /**
     * The error message for if the scene load failed.
     */
    val error: String? = null

    /**
     * Switches the scene to the requested scene.
     *
     * This function assumes that you have checked its progress and has checked if
     * it has succeeded or failed, and whether it is ready to be switched.
     *
     * If not checked, it will throw a [PrematureSceneSwitchException], even if [exceptionOnError]
     * is enabled or not.
     */
    fun switchTo(engine: DropbearEngine) {
        engine.native.switchToSceneAsync(this)
    }

    /**
     * Returns the progress of scene load.
     */
    fun progress(engine: DropbearEngine): Progress {
        return engine.native.getSceneLoadProgress(this)
    }

    /**
     * Checks if the scene load has completed.
     *
     * If completed, it will return true. If not, it will return false.
     *
     * If completed, it is recommended that you queue up the switch with [switchTo].
     */
    fun isComplete(): Boolean {
        return result == SceneLoadStatus.READY
    }

    /**
     * Checks if the scene load has failed.
     *
     * If failed, it will return true. If not, it will return false.
     *
     * If failed, it is recommended that you handle the error with [error].
     */
    fun hasFailed(): Boolean {
        return result == SceneLoadStatus.FAILED
    }

    /**
     * Returns the raw id of the handle.
     */
    fun raw(): Long {
        return id
    }
}