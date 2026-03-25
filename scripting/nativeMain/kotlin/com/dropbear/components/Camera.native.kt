@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import kotlin.String
import com.dropbear.math.Vector3d
import kotlinx.cinterop.*

internal actual fun Camera.getCameraEye(entity: EntityId): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val out = alloc<NVector3>()
    dropbear_camera_get_eye(world, entity.raw.toULong(), out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun Camera.setCameraEye(entity: EntityId, value: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val v = allocVec3(value)
    dropbear_camera_set_eye(world, entity.raw.toULong(), v.ptr)
}

internal actual fun Camera.getCameraTarget(entity: EntityId): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val out = alloc<NVector3>()
    dropbear_camera_get_target(world, entity.raw.toULong(), out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun Camera.setCameraTarget(entity: EntityId, value: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val v = allocVec3(value)
    dropbear_camera_set_target(world, entity.raw.toULong(), v.ptr)
}

internal actual fun Camera.getCameraUp(entity: EntityId): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val out = alloc<NVector3>()
    dropbear_camera_get_up(world, entity.raw.toULong(), out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun Camera.setCameraUp(entity: EntityId, value: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val v = allocVec3(value)
    dropbear_camera_set_up(world, entity.raw.toULong(), v.ptr)
}

internal actual fun Camera.getCameraAspect(entity: EntityId): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 1.0
    val out = alloc<DoubleVar>()
    dropbear_camera_get_aspect(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Camera.getCameraFovY(entity: EntityId): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 60.0
    val out = alloc<DoubleVar>()
    dropbear_camera_get_fovy(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Camera.setCameraFovY(entity: EntityId, value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_camera_set_fovy(world, entity.raw.toULong(), value)
}

internal actual fun Camera.getCameraZNear(entity: EntityId): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.1
    val out = alloc<DoubleVar>()
    dropbear_camera_get_znear(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Camera.setCameraZNear(entity: EntityId, value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_camera_set_znear(world, entity.raw.toULong(), value)
}

internal actual fun Camera.getCameraZFar(entity: EntityId): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 1000.0
    val out = alloc<DoubleVar>()
    dropbear_camera_get_zfar(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Camera.setCameraZFar(entity: EntityId, value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_camera_set_zfar(world, entity.raw.toULong(), value)
}

internal actual fun Camera.getCameraYaw(entity: EntityId): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.0
    val out = alloc<DoubleVar>()
    dropbear_camera_get_yaw(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Camera.getCameraPitch(entity: EntityId): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.0
    val out = alloc<DoubleVar>()
    dropbear_camera_get_pitch(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Camera.setCameraYaw(entity: EntityId, value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_camera_set_yaw(world, entity.raw.toULong(), value)
}

internal actual fun Camera.setCameraPitch(entity: EntityId, value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_camera_set_pitch(world, entity.raw.toULong(), value)
}

internal actual fun Camera.getCameraSpeed(entity: EntityId): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 1.0
    val out = alloc<DoubleVar>()
    dropbear_camera_get_speed(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Camera.setCameraSpeed(entity: EntityId, value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_camera_set_speed(world, entity.raw.toULong(), value)
}

internal actual fun Camera.getCameraSensitivity(entity: EntityId): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 1.0
    val out = alloc<DoubleVar>()
    dropbear_camera_get_sensitivity(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Camera.setCameraSensitivity(entity: EntityId, value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_camera_set_sensitivity(world, entity.raw.toULong(), value)
}

internal actual fun cameraExistsForEntity(entity: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_camera_exists_for_entity(world, entity.raw.toULong(), out.ptr)
    out.value
}