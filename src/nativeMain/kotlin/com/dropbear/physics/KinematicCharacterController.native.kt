package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.math.Vector3d

internal actual fun KinematicCharacterController.moveCharacter(dt: Double, translation: Vector3d) {
}

internal actual fun kccExistsForEntity(entityId: EntityId): Boolean {
    TODO("Not yet implemented")
}

internal actual fun KinematicCharacterController.getHitsNative(): List<CharacterCollision> {
    TODO("Not yet implemented")
}