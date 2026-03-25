@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityRef
import com.dropbear.ffi.generated.NShapeCastHit
import com.dropbear.ffi.generated.NVector3
import com.dropbear.ffi.generated.RayHit as FfiRayHit
import com.dropbear.ffi.generated.allocCollider
import com.dropbear.ffi.generated.allocColliderShape
import com.dropbear.ffi.generated.allocVec3
import com.dropbear.ffi.generated.dropbear_physics_get_gravity
import com.dropbear.ffi.generated.dropbear_physics_is_overlapping
import com.dropbear.ffi.generated.dropbear_physics_is_touching
import com.dropbear.ffi.generated.dropbear_physics_is_triggering
import com.dropbear.ffi.generated.dropbear_physics_raycast
import com.dropbear.ffi.generated.dropbear_physics_set_gravity
import com.dropbear.ffi.generated.dropbear_physics_shape_cast
import com.dropbear.ffi.generated.readCollider
import com.dropbear.ffi.generated.readShapeCastStatus
import com.dropbear.math.Vector3d
import kotlinx.cinterop.*

internal actual fun getGravity(): Vector3d = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped Vector3d.zero()
    val out = alloc<NVector3>()
    dropbear_physics_get_gravity(physics, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun setGravity(gravity: Vector3d) = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nv = allocVec3(gravity)
    dropbear_physics_set_gravity(physics, nv.ptr)
}

internal actual fun raycast(origin: Vector3d, direction: Vector3d, toi: Double, solid: Boolean): RayHit? = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped null
    val nOrigin = allocVec3(origin)
    val nDir = allocVec3(direction)
    val out = alloc<FfiRayHit>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_physics_raycast(physics, nOrigin.ptr, nDir.ptr, toi, solid, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else RayHit(readCollider(out.collider), out.distance)
}

internal actual fun isOverlapping(collider1: Collider, collider2: Collider): Boolean = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped false
    val nc1 = allocCollider(collider1)
    val nc2 = allocCollider(collider2)
    val out = alloc<BooleanVar>()
    dropbear_physics_is_overlapping(physics, nc1.ptr, nc2.ptr, out.ptr)
    out.value
}

internal actual fun isTriggering(collider1: Collider, collider2: Collider): Boolean = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped false
    val nc1 = allocCollider(collider1)
    val nc2 = allocCollider(collider2)
    val out = alloc<BooleanVar>()
    dropbear_physics_is_triggering(physics, nc1.ptr, nc2.ptr, out.ptr)
    out.value
}

internal actual fun isTouching(entity1: EntityRef, entity2: EntityRef): Boolean = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_physics_is_touching(physics, entity1.id.raw.toULong(), entity2.id.raw.toULong(), out.ptr)
    out.value
}

internal actual fun shapeCast(origin: Vector3d, direction: Vector3d, shape: ColliderShape, toi: Double, solid: Boolean): ShapeCastHit? = memScoped {
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped null
    val nOrigin = allocVec3(origin)
    val nDir = allocVec3(direction)
    val nShape = allocColliderShape(shape)
    val out = alloc<NShapeCastHit>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_physics_shape_cast(physics, nOrigin.ptr, nDir.ptr, nShape.ptr, toi, solid, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else ShapeCastHit(
        readCollider(out.collider),
        out.distance,
        Vector3d(out.witness1.x, out.witness1.y, out.witness1.z),
        Vector3d(out.witness2.x, out.witness2.y, out.witness2.z),
        Vector3d(out.normal1.x, out.normal1.y, out.normal1.z),
        Vector3d(out.normal2.x, out.normal2.y, out.normal2.z),
        readShapeCastStatus(out.status),
    )
}