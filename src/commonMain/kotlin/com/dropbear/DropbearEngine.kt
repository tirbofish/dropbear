package com.dropbear

import com.dropbear.asset.AssetHandle
import com.dropbear.ffi.NativeEngine
import com.dropbear.input.InputState
import com.dropbear.logging.Logger
import com.dropbear.scene.SceneManager
import com.dropbear.ui.UIManager

internal var exceptionOnError: Boolean = false
var lastErrorMessage: String? = null

/**
 * The main engine. 
 * 
 * All queries and fetching of entities run through this instance,
 * which contains a [NativeEngine] (contains native functions). 
 */
class DropbearEngine(val native: NativeEngine) {
    private var inputState: InputState? = null
    private var sceneManager: SceneManager? = null
    private var uiManager: UIManager? = null

    companion object {
        fun getLastErrMsg(): String? {
            return lastErrorMessage
        }

        /**
         * Globally sets whether exceptions should be thrown when an error occurs.
         *
         * This can be run in your update loop without consequences.
         */
        fun callExceptionOnError(toggle: Boolean) {
            exceptionOnError = toggle
        }
    }

    /**
     * Fetches an [EntityRef] with the given label.
     */
    fun getEntity(label: String): EntityRef? {
        val entityId = native.getEntity(label)
        val entityRef = if (entityId != null) EntityRef(EntityId(entityId)) else null
        entityRef?.engine = this
        return entityRef
    }

    /**
     * Fetches the information of the camera with the given label.
     */
    fun getCamera(label: String): Camera? {
        val result = native.getCamera(label)
        if (result != null) {
            result.engine = this
        }
        return result
    }

    /**
     * Gets the current [InputState] for that frame.
     */
    fun getInputState(): InputState {
        if (this.inputState == null) {
            Logger.trace("InputState not initialised, creating new one")
            this.inputState = InputState(native)
        }
        return this.inputState!!
    }

    /**
     * Gets the current [SceneManager] for that frame.
     */
    fun getSceneManager(): SceneManager {
        if (this.sceneManager == null) {
            Logger.trace("SceneManager not initialised, creating new one")
            this.sceneManager = SceneManager(native)
        }
        return this.sceneManager!!
    }

    /**
     * Gets the current [UIManager] for that frame.
     */
    fun getUIManager(): UIManager {
        if (this.uiManager == null) {
            Logger.trace("UiManager not initialised, creating new one")
            this.uiManager = UIManager(native)
        }
        return this.uiManager!!
    }

    /**
     * Fetches the asset information from the internal AssetRegistry (located in
     * `dropbear_engine::asset::AssetRegistry`).
     *
     * ## Warning
     * The eucalyptus asset URI (or `euca://`) is case-sensitive.
     */
    fun getAsset(eucaURI: String): AssetHandle? {
        val id = native.getAsset(eucaURI)
        return if (id != null) AssetHandle(id) else null
    }

    /**
     * Globally sets whether exceptions should be thrown when an error occurs.
     *
     * This can be run in your update loop without consequences.
     */
    fun callExceptionOnError(toggle: Boolean) = DropbearEngine.callExceptionOnError(toggle)

    /**
     * Quits the currently running app or game.
     * 
     * This function can have different behaviours depending on where it is ran. 
     * - eucalyptus-editor - When called, this exits your Play Mode session and returns you back to
     *                       `EditorState::Editing`
     * - redback-runtime - When called, this will exit your current process and kill the app as is. It will
     *                     also drop any pointers and do any additional cleanup.
     */
    fun quit() {
        native.quit()
    }
}