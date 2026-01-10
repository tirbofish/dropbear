package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.math.Transform
import com.dropbear.math.Vector3d

internal actual fun CharacterCollision.getCollider(): Collider {
    return CharacterCollisionNative.getCollider(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getCharacterPosition(): Transform {
    return CharacterCollisionNative.getCharacterPosition(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getTranslationApplied(): Vector3d {
    return CharacterCollisionNative.getTranslationApplied(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getTranslationRemaining(): Vector3d {
    return CharacterCollisionNative.getTranslationRemaining(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getTimeOfImpact(): Double {
    return CharacterCollisionNative.getTimeOfImpact(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getWitness1(): Vector3d {
    return CharacterCollisionNative.getWitness1(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getWitness2(): Vector3d {
    return CharacterCollisionNative.getWitness2(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getNormal1(): Vector3d {
    return CharacterCollisionNative.getNormal1(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getNormal2(): Vector3d {
    return CharacterCollisionNative.getNormal2(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}

internal actual fun CharacterCollision.getStatus(): ShapeCastStatus {
    return CharacterCollisionNative.getStatus(DropbearEngine.native.worldHandle, entity.raw, collisionHandle)
}