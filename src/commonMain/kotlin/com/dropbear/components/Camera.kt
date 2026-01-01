package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3d

/**
 * Describes a 3D camera, as defined in `dropbear_engine::camera::Camera`
 */
class Camera(
    internal val entity: EntityId,
): Component(entity, "Camera3D") {
    /**
     * The eye/position of the camera.
     */
    var eye: Vector3d
        get() = getCameraEye(entity)
        set(value) = setCameraEye(entity, value)

    /**
     * The target of the entity/the direction it is looking at.
     */
    var target: Vector3d
        get() = getCameraTarget(entity)
        set(value) = setCameraTarget(entity, value)

    /**
     * The up value.
     *
     * Default: [Vector3d.up]
     */
    var up: Vector3d
        get() = getCameraUp(entity)
        set(value) = setCameraUp(entity, value)

    /**
     * The aspect ratio of the camera.
     *
     * Often this is set from the window size. It cannot be overridden.
     */
    val aspect: Double
        get() = getCameraAspect(entity)

    /**
     * The horizontal FOV value.
     */
    var fovY: Double
        get() = getCameraFovY(entity)
        set(value) = setCameraFovY(entity, value)

    /**
     * The nearest distance the camera is able to see.
     *
     * Default: `0.1`
     */
    var znear: Double
        get() = getCameraZNear(entity)
        set(value) = setCameraZNear(entity, value)

    /**
     * The nearest distance the camera is able to see.
     *
     * Default: `100.0`
     */
    var zfar: Double
        get() = getCameraZFar(entity)
        set(value) = setCameraZFar(entity, value)

    /**
     * The horizontal rotational angle.
     *
     * # Calculation
     * ```rust
     * let dir = (builder.target - builder.eye).normalize();
     * let yaw = dir.z.atan2(dir.x);
     * ```
     */
    var yaw: Double
        get() = getCameraYaw(entity)
        set(value) = setCameraYaw(entity, value)

    /**
     * The vertical rotational angle.
     *
     * # Calculation
     * ```rust
     * let dir = (builder.target - builder.eye).normalize();
     * let pitch = dir.y.clamp(-1.0, 1.0).asin();
     * ```
     */
    var pitch: Double
        get() = getCameraPitch(entity)
        set(value) = setCameraPitch(entity, value)

    /**
     * The movement speed of the camera.
     *
     * Default: `1.0`
     */
    var speed: Double
        get() = getCameraSpeed(entity)
        set(value) = setCameraSpeed(entity, value)

    /**
     * The sensitivity of the mouse.
     *
     * Default: `0.002`
     */
    var sensitivity: Double
        get() = getCameraSensitivity(entity)
        set(value) = setCameraSensitivity(entity, value)

    override fun toString(): String {
        return "Camera component of entity $entity \n" +
                "eye: $eye\n" +
                "target: $target\n" +
                "up: $up\n " +
                "aspect: $aspect \n" +
                "fov_y: $fovY \n" +
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

expect fun Camera.getCameraEye(entity: EntityId): Vector3d
expect fun Camera.setCameraEye(entity: EntityId, value: Vector3d)
expect fun Camera.getCameraTarget(entity: EntityId): Vector3d
expect fun Camera.setCameraTarget(entity: EntityId, value: Vector3d)
expect fun Camera.getCameraUp(entity: EntityId): Vector3d
expect fun Camera.setCameraUp(entity: EntityId, value: Vector3d)
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