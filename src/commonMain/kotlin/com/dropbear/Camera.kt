package com.dropbear

import com.dropbear.math.Vector3D

/**
 * Describes a 3D camera, as defined in `dropbear_engine::camera::Camera`
 */
class Camera(
    internal val entity: EntityId,
) {
    var eye: Vector3D
        get() = native.getCameraEye(this)
        set(value) = native.setCameraEye(this, value)
    var target: Vector3D
        get() = native.getCameraTarget(this)
        set(value) = native.setCameraTarget(this, value)
    var up: Vector3D
        get() = native.getCameraUp(this)
        set(value) = native.setCameraUp(this, value)
    val aspect: Double
        get() = native.getCameraAspect(this)
    var fov_y: Double
        get() = native.getCameraFovY(this)
        set(value) = native.setCameraFovY(this, value)
    var znear: Double
        get() = native.getCameraZNear(this)
        set(value) = native.setCameraZNear(this, value)
    var zfar: Double
        get() = native.getCameraZFar(this)
        set(value) = native.setCameraZFar(this)
    var yaw: Double
        get() = native.getCameraYaw(this)
        set(value) = native.setCameraYaw(this, value)
    var pitch: Double
        get() = native.getCameraPitch(this)
        set(value) = native.setCameraPitch(this, value)
    var speed: Double
        get() = native.getCameraSpeed(this)
        set(value) = native.setCameraSpeed(this, value)
    var sensitivity: Double
        get() = native.getCameraSensitivity(this)
        set(value) = native.setCameraSensitivity(this, value)

    internal lateinit var engine: DropbearEngine

    override fun toString(): String {
        return "Camera '${label}' of id $id \n" +
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
}