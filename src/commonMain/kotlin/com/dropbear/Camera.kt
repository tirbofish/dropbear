package com.dropbear

import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3D

/**
 * Describes a 3D camera, as defined in `dropbear_engine::camera::Camera`
 */
class Camera(
    internal val entity: EntityId,
): Component(entity, "Camera3D") {
    var eye: Vector3D
        get() = getCameraEye(entity)
        set(value) = setCameraEye(entity, value)
    var target: Vector3D
        get() = getCameraTarget(entity)
        set(value) = setCameraTarget(entity, value)
    var up: Vector3D
        get() = getCameraUp(entity)
        set(value) = setCameraUp(entity, value)
    val aspect: Double
        get() = getCameraAspect(entity)
    var fov_y: Double
        get() = getCameraFovY(entity)
        set(value) = setCameraFovY(entity, value)
    var znear: Double
        get() = getCameraZNear(entity)
        set(value) = setCameraZNear(entity, value)
    var zfar: Double
        get() = getCameraZFar(entity)
        set(value) = setCameraZFar(entity, value)
    var yaw: Double
        get() = getCameraYaw(entity)
        set(value) = setCameraYaw(entity, value)
    var pitch: Double
        get() = getCameraPitch(entity)
        set(value) = setCameraPitch(entity, value)
    var speed: Double
        get() = getCameraSpeed(entity)
        set(value) = setCameraSpeed(entity, value)
    var sensitivity: Double
        get() = getCameraSensitivity(entity)
        set(value) = setCameraSensitivity(entity, value)

    override fun toString(): String {
        return "Camera component of entity $entity \n" +
                "eye: $eye\n" +
                "target: $target\n" +
                "up: $up\n " +
                "aspect: $aspect \n" +
                "fov_y: $fov_y \n" +
                "znear: $znear \n" +
                "zfar: $zfar \n" +
                "yaw: $yaw \n" +
                "pitch: $pitch" +
                "speed: $speed" +
                "sensitivity: $sensitivity"
    }

    companion object : ComponentType<Camera> {
        override fun get(entityId: EntityId): Camera? {
            return if (cameraExistsForEntity(entityId)) Camera(entityId) else null
        }
    }
}

expect fun Camera.getCameraEye(entity: EntityId): Vector3D
expect fun Camera.setCameraEye(entity: EntityId, value: Vector3D)
expect fun Camera.getCameraTarget(entity: EntityId): Vector3D
expect fun Camera.setCameraTarget(entity: EntityId, value: Vector3D)
expect fun Camera.getCameraUp(entity: EntityId): Vector3D
expect fun Camera.setCameraUp(entity: EntityId, value: Vector3D)
expect fun Camera.getCameraAspect(entity: EntityId): Double
expect fun Camera.getCameraFovY(entity: EntityId): Double
expect fun Camera.setCameraFovY(entity: EntityId, value: Double)
expect fun Camera.getCameraZNear(entity: EntityId): Double
expect fun Camera.setCameraZNear(entity: EntityId, value: Double)
expect fun Camera.getCameraZFar(entity: EntityId): Double
expect fun Camera.setCameraZFar(entity: EntityId, value: Double)
expect fun Camera.getCameraYaw(entity: EntityId): Double
expect fun Camera.setCameraYaw(entity: EntityId, value: Double)
expect fun Camera.getCameraPitch(entity: EntityId): Double
expect fun Camera.setCameraPitch(entity: EntityId, value: Double)
expect fun Camera.getCameraSpeed(entity: EntityId): Double
expect fun Camera.setCameraSpeed(entity: EntityId, value: Double)
expect fun Camera.getCameraSensitivity(entity: EntityId): Double
expect fun Camera.setCameraSensitivity(entity: EntityId, value: Double)

expect fun cameraExistsForEntity(entity: EntityId): Boolean