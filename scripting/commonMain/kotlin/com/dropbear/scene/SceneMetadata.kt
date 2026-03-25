package com.dropbear.scene

import com.dropbear.EntityRef

/**
 * A class that describes a `eucalyptus_core::scene::SceneConfig` and other information
 * related to that specific scene.
 */
class SceneMetadata(
    val id: Long,
    val name: String,
    val settings: SceneSettings
) {
    val entities: List<EntityRef>
        get() = getEntities()
}

expect fun SceneMetadata.getEntities(): List<EntityRef>