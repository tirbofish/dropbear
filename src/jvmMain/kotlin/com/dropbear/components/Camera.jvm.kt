package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.math.Vector3d

internal actual fun Camera.getCameraEye(entity: EntityId): Vector3d {
    return CameraNative.getCameraEye(DropbearEngine.native.worldHandle, entity.raw)
        ?: Vector3d.zero()
}

internal actual fun Camera.setCameraEye(entity: EntityId, value: Vector3d) {
    CameraNative.setCameraEye(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraTarget(entity: EntityId): Vector3d {
    return CameraNative.getCameraTarget(DropbearEngine.native.worldHandle, entity.raw)
        ?: Vector3d.zero()
}

internal actual fun Camera.setCameraTarget(entity: EntityId, value: Vector3d) {
    CameraNative.setCameraTarget(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraUp(entity: EntityId): Vector3d {
    return CameraNative.getCameraUp(DropbearEngine.native.worldHandle, entity.raw)
        ?: Vector3d.up()
}

internal actual fun Camera.setCameraUp(entity: EntityId, value: Vector3d) {
    CameraNative.setCameraUp(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraAspect(entity: EntityId): Double {
    return CameraNative.getCameraAspect(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Camera.getCameraFovY(entity: EntityId): Double {
    return CameraNative.getCameraFovY(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Camera.setCameraFovY(entity: EntityId, value: Double) {
    CameraNative.setCameraFovY(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraZNear(entity: EntityId): Double {
    return CameraNative.getCameraZNear(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Camera.setCameraZNear(entity: EntityId, value: Double) {
    CameraNative.setCameraZNear(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraZFar(entity: EntityId): Double {
    return CameraNative.getCameraZFar(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Camera.setCameraZFar(entity: EntityId, value: Double) {
    CameraNative.setCameraZFar(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraYaw(entity: EntityId): Double {
    return CameraNative.getCameraYaw(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Camera.setCameraYaw(entity: EntityId, value: Double) {
    CameraNative.setCameraYaw(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraPitch(entity: EntityId): Double {
    return CameraNative.getCameraPitch(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Camera.setCameraPitch(entity: EntityId, value: Double) {
    CameraNative.setCameraPitch(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraSpeed(entity: EntityId): Double {
    return CameraNative.getCameraSpeed(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Camera.setCameraSpeed(entity: EntityId, value: Double) {
    CameraNative.setCameraSpeed(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun Camera.getCameraSensitivity(entity: EntityId): Double {
    return CameraNative.getCameraSensitivity(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Camera.setCameraSensitivity(entity: EntityId, value: Double) {
    CameraNative.setCameraSensitivity(DropbearEngine.native.worldHandle, entity.raw, value)
}

internal actual fun cameraExistsForEntity(entity: EntityId): Boolean {
    return CameraNative.cameraExistsForEntity(DropbearEngine.native.worldHandle, entity.raw)
}