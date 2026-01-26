package com.dropbear

import com.dropbear.asset.AssetHandle
import com.dropbear.ffi.NativeEngine
import com.dropbear.input.InputState
import com.dropbear.logging.Logger
import com.dropbear.scene.SceneManager
import com.dropbear.ui.UIInstruction

internal var exceptionOnError: Boolean = false
var lastErrorMessage: String? = null

/**
 * The main engine. 
 * 
 * All queries and fetching of entities run through this instance,
 * which contains a [NativeEngine] (contains native functions). 
 */
class DropbearEngine(val native: NativeEngine) {
    val inputState: InputState = InputState()
    val sceneManager: SceneManager = SceneManager()

    init {
        Companion.native = native
    }

    companion object {
        lateinit var native: NativeEngine

        @Deprecated("Not implemented yet", level = DeprecationLevel.HIDDEN)
        fun getLastErrMsg(): String? {
            return lastErrorMessage
        }

        /**
         * Globally sets whether exceptions should be thrown when an error occurs.
         *
         * This can be run in your update loop without consequences.
         */
        @Deprecated("Currently not supported anymore, automatically throws exception on error. " +
                "Better to catch the exception instead", level = DeprecationLevel.HIDDEN)
        fun callExceptionOnError(toggle: Boolean) {
            exceptionOnError = toggle
        }
    }

    /**
     * Fetches an [EntityRef] with the given label.
     */
    fun getEntity(label: String): EntityRef? {
        val entityId = com.dropbear.getEntity(label)
        val entityRef = if (entityId != null) EntityRef(EntityId(entityId)) else null
        return entityRef
    }

    /**
     * Fetches the asset information from the internal AssetRegistry (located in
     * `dropbear_engine::asset::AssetRegistry`).
     *
     * ## Warning
     * The eucalyptus asset URI (or `euca://`) is case-sensitive.
     */
    fun getAsset(eucaURI: String): AssetHandle? {
        val id = com.dropbear.getAsset(eucaURI)
        return if (id != null) AssetHandle(id) else null
    }

    /**
     * Globally sets whether exceptions should be thrown when an error occurs.
     *
     * This can be run in your update loop without consequences.
     */
    @Deprecated("Currently not supported anymore, automatically throws exception on error. " +
            "Better to catch the exception instead", level = DeprecationLevel.HIDDEN)
    fun callExceptionOnError(toggle: Boolean) {
    }

    fun renderUI(uiInstructionSet: List<UIInstruction>) {
        Logger.trace("instructions: $uiInstructionSet")
        renderUI(instructions = uiInstructionSet)
    }

    /**
     * Quits the currently running app or game elegantly.
     * 
     * This function can have different behaviours depending on where it is ran. 
     * - eucalyptus-editor - When called, this exits your Play Mode session and returns you back to
     *                       `EditorState::Editing`
     * - redback-runtime - When called, this will exit your current process and kill the app as is. It will
     *                     also drop any pointers and do any additional cleanup.
     */
    fun quit() {
        com.dropbear.quit()
    }
}

internal expect fun getEntity(label: String): Long?
internal expect fun getAsset(eucaURI: String): Long?
internal expect fun quit()
internal expect fun renderUI(instructions: List<UIInstruction>)