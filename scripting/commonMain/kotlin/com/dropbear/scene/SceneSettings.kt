package com.dropbear.scene

class SceneSettings {
    /**
     * Returns whether the scene's assets has been preloaded.
     */
    // must be val, cannot be changed (how could it possibly do anything if changed)
    val preloaded: Boolean
        get() = getPreload()

    /**
     * Shows hitboxes/wireframe for colliders for that specific scene.
     */
    var showHitboxes: Boolean
        get() = getHitboxState()
        set(value) = setHitboxState(value)

    /**
     * Overlays the HUD on top of the viewport if it exists as an entity.
     */
    var overlayHUD: Boolean
        get() = getOverlayHUDState()
        set(value) = setOverlayHUDState(value)

    /**
     * Overlays all billboard ui for all entities if it exists as a component.
     */
    var overlayBillboard: Boolean
        get() = getOverlayBillboardState()
        set(value) = setOverlayBillboardState(value)

    /**
     * Controls the strength of ambient/IBL lighting for this scene.
     */
    var ambientStrength: Double
        get() = getAmbientStrength()
        set(value) = setAmbientStrength(value)
}

expect fun SceneSettings.getPreload(): Boolean
expect fun SceneSettings.getHitboxState(): Boolean
expect fun SceneSettings.setHitboxState(value: Boolean)
expect fun SceneSettings.getOverlayHUDState(): Boolean
expect fun SceneSettings.setOverlayHUDState(value: Boolean)
expect fun SceneSettings.getOverlayBillboardState(): Boolean
expect fun SceneSettings.setOverlayBillboardState(value: Boolean)
expect fun SceneSettings.getAmbientStrength(): Double
expect fun SceneSettings.setAmbientStrength(value: Double)