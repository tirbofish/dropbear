package com.dropbear.exception

import com.dropbear.scene.SceneLoadHandle

/**
 * Exception thrown when a [SceneLoadHandle.switchTo] is called before the scene is loaded.
 *
 * The specific error code is `-10` and is shown in
 * `eucalyptus_core::scripting::native::DropbearNativeError::PrematureSceneSwitch`
 */
class PrematureSceneSwitchException(message: String? = null, cause: Throwable? = null): Exception(message, cause)
