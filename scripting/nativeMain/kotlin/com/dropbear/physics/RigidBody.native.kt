@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.AxisLock as FfiAxisLock
import com.dropbear.ffi.generated.NColliderArray
import com.dropbear.ffi.generated.NVector3
import com.dropbear.ffi.generated.RigidBodyContext
import com.dropbear.ffi.generated.allocCollider
import com.dropbear.ffi.generated.allocRigidBodyCtx
import com.dropbear.ffi.generated.dropbear_rigidbody_apply_impulse
import com.dropbear.ffi.generated.dropbear_rigidbody_apply_torque_impulse
import com.dropbear.ffi.generated.dropbear_rigidbody_exists_for_entity
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_angular_damping
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_angular_velocity
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_ccd_enabled
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_children
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_gravity_scale
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_linear_damping
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_linear_velocity
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_lock_rotation
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_lock_translation
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_mode
import com.dropbear.ffi.generated.dropbear_rigidbody_get_rigidbody_sleep
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_angular_damping
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_angular_velocity
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_ccd_enabled
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_gravity_scale
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_linear_damping
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_linear_velocity
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_lock_rotation
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_lock_translation
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_mode
import com.dropbear.ffi.generated.dropbear_rigidbody_set_rigidbody_sleep
import com.dropbear.ffi.generated.IndexNative
import com.dropbear.ffi.generated.readCollider
import com.dropbear.math.Vector3d
import kotlinx.cinterop.*

private fun MemScope.rbCtx(rb: RigidBody): RigidBodyContext = allocRigidBodyCtx(rb)

internal actual fun RigidBody.getRigidbodyMode(rigidBody: RigidBody): RigidBodyMode = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped RigidBodyMode.Dynamic
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped RigidBodyMode.Dynamic
    val ctx = rbCtx(rigidBody)
    val out = alloc<IntVar>()
    dropbear_rigidbody_get_rigidbody_mode(world, physics, ctx.ptr, out.ptr)
    RigidBodyMode.entries[out.value.coerceIn(0, RigidBodyMode.entries.lastIndex)]
}

internal actual fun RigidBody.setRigidbodyMode(rigidBody: RigidBody, mode: RigidBodyMode) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val ctx = rbCtx(rigidBody)
    dropbear_rigidbody_set_rigidbody_mode(world, physics, ctx.ptr, mode.ordinal)
}

internal actual fun RigidBody.getRigidbodyGravityScale(rigidBody: RigidBody): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 1.0
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped 1.0
    val ctx = rbCtx(rigidBody)
    val out = alloc<DoubleVar>()
    dropbear_rigidbody_get_rigidbody_gravity_scale(world, physics, ctx.ptr, out.ptr)
    out.value
}

internal actual fun RigidBody.setRigidbodyGravityScale(rigidBody: RigidBody, gravityScale: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    dropbear_rigidbody_set_rigidbody_gravity_scale(world, physics, rbCtx(rigidBody).ptr, gravityScale)
}

internal actual fun RigidBody.getRigidBodySleep(rigidBody: RigidBody): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_rigidbody_get_rigidbody_sleep(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    out.value
}

internal actual fun RigidBody.setRigidBodySleep(rigidBody: RigidBody, canSleep: Boolean) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    dropbear_rigidbody_set_rigidbody_sleep(world, physics, rbCtx(rigidBody).ptr, canSleep)
}

internal actual fun RigidBody.getRigidbodyCcdEnabled(rigidBody: RigidBody): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_rigidbody_get_rigidbody_ccd_enabled(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    out.value
}

internal actual fun RigidBody.setRigidbodyCcdEnabled(rigidBody: RigidBody, ccdEnabled: Boolean) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    dropbear_rigidbody_set_rigidbody_ccd_enabled(world, physics, rbCtx(rigidBody).ptr, ccdEnabled)
}

internal actual fun RigidBody.getRigidbodyLinearVelocity(rigidBody: RigidBody): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped Vector3d.zero()
    val out = alloc<NVector3>()
    dropbear_rigidbody_get_rigidbody_linear_velocity(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun RigidBody.setRigidbodyLinearVelocity(rigidBody: RigidBody, linearVelocity: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nv = alloc<NVector3>().also { it.x = linearVelocity.x; it.y = linearVelocity.y; it.z = linearVelocity.z }
    dropbear_rigidbody_set_rigidbody_linear_velocity(world, physics, rbCtx(rigidBody).ptr, nv.ptr)
}

internal actual fun RigidBody.getRigidbodyLinearDamping(rigidBody: RigidBody): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.0
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped 0.0
    val out = alloc<DoubleVar>()
    dropbear_rigidbody_get_rigidbody_linear_damping(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    out.value
}

internal actual fun RigidBody.setRigidbodyLinearDamping(rigidBody: RigidBody, linearDamping: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    dropbear_rigidbody_set_rigidbody_linear_damping(world, physics, rbCtx(rigidBody).ptr, linearDamping)
}

internal actual fun RigidBody.getRigidbodyAngularVelocity(rigidBody: RigidBody): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped Vector3d.zero()
    val out = alloc<NVector3>()
    dropbear_rigidbody_get_rigidbody_angular_velocity(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun RigidBody.setRigidbodyAngularVelocity(rigidBody: RigidBody, angularVelocity: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nv = alloc<NVector3>().also { it.x = angularVelocity.x; it.y = angularVelocity.y; it.z = angularVelocity.z }
    dropbear_rigidbody_set_rigidbody_angular_velocity(world, physics, rbCtx(rigidBody).ptr, nv.ptr)
}

internal actual fun RigidBody.getRigidbodyAngularDamping(rigidBody: RigidBody): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.0
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped 0.0
    val out = alloc<DoubleVar>()
    dropbear_rigidbody_get_rigidbody_angular_damping(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    out.value
}

internal actual fun RigidBody.setRigidbodyAngularDamping(rigidBody: RigidBody, angularDamping: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    dropbear_rigidbody_set_rigidbody_angular_damping(world, physics, rbCtx(rigidBody).ptr, angularDamping)
}

internal actual fun RigidBody.getRigidbodyLockTranslation(rigidBody: RigidBody): AxisLock = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped AxisLock()
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped AxisLock()
    val out = alloc<FfiAxisLock>()
    dropbear_rigidbody_get_rigidbody_lock_translation(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    AxisLock(out.x, out.y, out.z)
}

internal actual fun RigidBody.setRigidbodyLockTranslation(rigidBody: RigidBody, lockTranslation: AxisLock) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val al = alloc<FfiAxisLock>().also { it.x = lockTranslation.x; it.y = lockTranslation.y; it.z = lockTranslation.z }
    dropbear_rigidbody_set_rigidbody_lock_translation(world, physics, rbCtx(rigidBody).ptr, al.ptr)
}

internal actual fun RigidBody.getRigidbodyLockRotation(rigidBody: RigidBody): AxisLock = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped AxisLock()
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped AxisLock()
    val out = alloc<FfiAxisLock>()
    dropbear_rigidbody_get_rigidbody_lock_rotation(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    AxisLock(out.x, out.y, out.z)
}

internal actual fun RigidBody.setRigidbodyLockRotation(rigidBody: RigidBody, lockRotation: AxisLock) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val al = alloc<FfiAxisLock>().also { it.x = lockRotation.x; it.y = lockRotation.y; it.z = lockRotation.z }
    dropbear_rigidbody_set_rigidbody_lock_rotation(world, physics, rbCtx(rigidBody).ptr, al.ptr)
}

internal actual fun RigidBody.getRigidbodyChildren(rigidBody: RigidBody): List<Collider> = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped emptyList()
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped emptyList()
    val out = alloc<NColliderArray>()
    val rc = dropbear_rigidbody_get_rigidbody_children(world, physics, rbCtx(rigidBody).ptr, out.ptr)
    if (rc != 0) return@memScoped emptyList()
    val ptr = out.values ?: return@memScoped emptyList()
    val len = out.length.toInt()
    (0 until len).map { i -> readCollider(ptr[i]) }
}

internal actual fun RigidBody.applyImpulse(index: Index, x: Double, y: Double, z: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val ctx = alloc<RigidBodyContext>()
    ctx.index.index = index.index; ctx.index.generation = index.generation
    ctx.entity_id = this@applyImpulse.entity.raw.toULong()
    dropbear_rigidbody_apply_impulse(world, physics, ctx.ptr, x, y, z)
}

internal actual fun RigidBody.applyTorqueImpulse(index: Index, x: Double, y: Double, z: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val ctx = alloc<RigidBodyContext>()
    ctx.index.index = index.index; ctx.index.generation = index.generation
    ctx.entity_id = this@applyTorqueImpulse.entity.raw.toULong()
    dropbear_rigidbody_apply_torque_impulse(world, physics, ctx.ptr, x, y, z)
}

internal actual fun rigidBodyExistsForEntity(entityId: EntityId): Index? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped null
    val out = alloc<IndexNative>()
    val present = alloc<BooleanVar>()
    dropbear_rigidbody_exists_for_entity(world, physics, entityId.raw.toULong(), out.ptr, present.ptr)
    if (!present.value) null else Index(out.index, out.generation)
}