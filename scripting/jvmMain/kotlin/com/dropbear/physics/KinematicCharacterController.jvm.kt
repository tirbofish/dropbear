package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.math.Vector3d

internal actual fun kccExistsForEntity(entityId: EntityId): Boolean {
    return KinematicCharacterControllerNative.existsForEntity(DropbearEngine.native.worldHandle, entityId.raw)
}

internal actual fun KinematicCharacterController.moveCharacter(dt: Double, translation: Vector3d) {
    return KinematicCharacterControllerNative.moveCharacter(
        DropbearEngine.native.worldHandle,
        DropbearEngine.native.physicsEngineHandle,
        entity.raw,
        translation,
        dt
    )
}

internal actual fun KinematicCharacterController.setRotationNative(rotation: com.dropbear.math.Quaterniond) {
    return KinematicCharacterControllerNative.setRotation(
        DropbearEngine.native.worldHandle,
        DropbearEngine.native.physicsEngineHandle,
        entity.raw,
        rotation
    )
}

internal actual fun KinematicCharacterController.getHitsNative(): List<CharacterCollision> {
    return KinematicCharacterControllerNative.getHitNative(
        DropbearEngine.native.worldHandle,
        entity.raw,
    ).toList()
}