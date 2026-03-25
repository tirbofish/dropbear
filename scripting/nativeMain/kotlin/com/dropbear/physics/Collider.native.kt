@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.ffi.generated.ColliderShapeFfi
import com.dropbear.ffi.generated.NVector3
import com.dropbear.ffi.generated.allocCollider
import com.dropbear.ffi.generated.allocColliderShape
import com.dropbear.ffi.generated.allocVec3
import com.dropbear.ffi.generated.dropbear_collider_get_collider_density
import com.dropbear.ffi.generated.dropbear_collider_get_collider_friction
import com.dropbear.ffi.generated.dropbear_collider_get_collider_is_sensor
import com.dropbear.ffi.generated.dropbear_collider_get_collider_mass
import com.dropbear.ffi.generated.dropbear_collider_get_collider_restitution
import com.dropbear.ffi.generated.dropbear_collider_get_collider_rotation
import com.dropbear.ffi.generated.dropbear_collider_get_collider_shape
import com.dropbear.ffi.generated.dropbear_collider_get_collider_translation
import com.dropbear.ffi.generated.dropbear_collider_set_collider_density
import com.dropbear.ffi.generated.dropbear_collider_set_collider_friction
import com.dropbear.ffi.generated.dropbear_collider_set_collider_is_sensor
import com.dropbear.ffi.generated.dropbear_collider_set_collider_mass
import com.dropbear.ffi.generated.dropbear_collider_set_collider_restitution
import com.dropbear.ffi.generated.dropbear_collider_set_collider_rotation
import com.dropbear.ffi.generated.dropbear_collider_set_collider_shape
import com.dropbear.ffi.generated.dropbear_collider_set_collider_translation
import com.dropbear.ffi.generated.readColliderShape
import com.dropbear.math.Vector3d
import kotlinx.cinterop.*

internal actual fun Collider.getColliderShape(collider: Collider): ColliderShape = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped ColliderShape.Box(Vector3d.zero())
    val nc = allocCollider(collider)
    val out = alloc<ColliderShapeFfi>()
    dropbear_collider_get_collider_shape(physics, nc.ptr, out.ptr)
    readColliderShape(out)
}

internal actual fun Collider.setColliderShape(collider: Collider, shape: ColliderShape) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nc = allocCollider(collider)
    val ns = allocColliderShape(shape)
    dropbear_collider_set_collider_shape(physics, nc.ptr, ns.ptr)
}

internal actual fun Collider.setColliderDensity(collider: Collider, density: Double) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nc = allocCollider(collider)
    dropbear_collider_set_collider_density(physics, nc.ptr, density)
}

internal actual fun Collider.getColliderFriction(collider: Collider): Double = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped 0.0
    val nc = allocCollider(collider)
    val out = alloc<DoubleVar>()
    dropbear_collider_get_collider_friction(physics, nc.ptr, out.ptr)
    out.value
}

internal actual fun Collider.setColliderFriction(collider: Collider, friction: Double) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nc = allocCollider(collider)
    dropbear_collider_set_collider_friction(physics, nc.ptr, friction)
}

internal actual fun Collider.getColliderRestitution(collider: Collider): Double = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped 0.0
    val nc = allocCollider(collider)
    val out = alloc<DoubleVar>()
    dropbear_collider_get_collider_restitution(physics, nc.ptr, out.ptr)
    out.value
}

internal actual fun Collider.setColliderRestitution(collider: Collider, restitution: Double) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nc = allocCollider(collider)
    dropbear_collider_set_collider_restitution(physics, nc.ptr, restitution)
}

internal actual fun Collider.getColliderIsSensor(collider: Collider): Boolean = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped false
    val nc = allocCollider(collider)
    val out = alloc<BooleanVar>()
    dropbear_collider_get_collider_is_sensor(physics, nc.ptr, out.ptr)
    out.value
}

internal actual fun Collider.setColliderIsSensor(collider: Collider, isSensor: Boolean) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nc = allocCollider(collider)
    dropbear_collider_set_collider_is_sensor(physics, nc.ptr, isSensor)
}

internal actual fun Collider.getColliderTranslation(collider: Collider): Vector3d = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped Vector3d.zero()
    val nc = allocCollider(collider)
    val out = alloc<NVector3>()
    dropbear_collider_get_collider_translation(physics, nc.ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun Collider.setColliderTranslation(collider: Collider, translation: Vector3d) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nc = allocCollider(collider)
    val nv = allocVec3(translation)
    dropbear_collider_set_collider_translation(physics, nc.ptr, nv.ptr)
}

internal actual fun Collider.getColliderRotation(collider: Collider): Vector3d = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped Vector3d.zero()
    val nc = allocCollider(collider)
    val out = alloc<NVector3>()
    dropbear_collider_get_collider_rotation(physics, nc.ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun Collider.setColliderRotation(collider: Collider, rotation: Vector3d) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nc = allocCollider(collider)
    val nv = allocVec3(rotation)
    dropbear_collider_set_collider_rotation(physics, nc.ptr, nv.ptr)
}

internal actual fun Collider.getColliderMass(collider: Collider): Double = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped 0.0
    val nc = allocCollider(collider)
    val out = alloc<DoubleVar>()
    dropbear_collider_get_collider_mass(physics, nc.ptr, out.ptr)
    out.value
}

internal actual fun Collider.setColliderMass(collider: Collider, mass: Double) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nc = allocCollider(collider)
    dropbear_collider_set_collider_mass(physics, nc.ptr, mass)
}

internal actual fun Collider.getColliderDensity(collider: Collider): Double = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped 0.0
    val nc = allocCollider(collider)
    val out = alloc<DoubleVar>()
    dropbear_collider_get_collider_density(physics, nc.ptr, out.ptr)
    out.value
}
