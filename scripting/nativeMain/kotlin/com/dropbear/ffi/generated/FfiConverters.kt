@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.ffi.generated

import com.dropbear.EntityId
import com.dropbear.math.Quaterniond
import com.dropbear.math.Transform
import com.dropbear.math.Vector3d
import com.dropbear.physics.Collider
import com.dropbear.physics.ColliderShape
import com.dropbear.physics.Index
import com.dropbear.physics.RigidBody
import com.dropbear.physics.ShapeCastStatus
import com.dropbear.utils.Colour
import kotlinx.cinterop.*

internal fun readTransform(nt: NTransform): Transform = Transform(
    Vector3d(nt.position.x, nt.position.y, nt.position.z),
    Quaterniond(nt.rotation.x, nt.rotation.y, nt.rotation.z, nt.rotation.w),
    Vector3d(nt.scale.x, nt.scale.y, nt.scale.z),
)

internal fun MemScope.allocTransform(t: Transform): NTransform {
    val nt = alloc<NTransform>()
    nt.position.x = t.position.x
    nt.position.y = t.position.y
    nt.position.z = t.position.z
    nt.rotation.x = t.rotation.x
    nt.rotation.y = t.rotation.y
    nt.rotation.z = t.rotation.z
    nt.rotation.w = t.rotation.w
    nt.scale.x = t.scale.x
    nt.scale.y = t.scale.y
    nt.scale.z = t.scale.z
    return nt
}

internal fun MemScope.allocVec3(v: Vector3d): NVector3 {
    val nv = alloc<NVector3>()
    nv.x = v.x; nv.y = v.y; nv.z = v.z
    return nv
}

internal fun MemScope.allocQuat(q: Quaterniond): NQuaternion {
    val nq = alloc<NQuaternion>()
    nq.x = q.x; nq.y = q.y; nq.z = q.z; nq.w = q.w
    return nq
}

internal fun readCollider(nc: NCollider): Collider = Collider(
    Index(nc.index.index, nc.index.generation),
    EntityId(nc.entity_id.toLong()),
    nc.id,
)

internal fun MemScope.allocCollider(c: Collider): NCollider {
    val nc = alloc<NCollider>()
    nc.index.index = c.index.index
    nc.index.generation = c.index.generation
    nc.entity_id = c.entity.raw.toULong()
    nc.id = c.id
    return nc
}

internal fun MemScope.allocRigidBodyCtx(rb: RigidBody): RigidBodyContext {
    val ctx = alloc<RigidBodyContext>()
    ctx.index.index = rb.index.index
    ctx.index.generation = rb.index.generation
    ctx.entity_id = rb.entity.raw.toULong()
    return ctx
}

internal fun MemScope.allocIndexNative(idx: Index): IndexNative {
    val ni = alloc<IndexNative>()
    ni.index = idx.index
    ni.generation = idx.generation
    return ni
}

internal fun readColour(nc: NColour): Colour = Colour(nc.r, nc.g, nc.b, nc.a)

internal fun MemScope.allocColour(c: Colour): NColour {
    val nc = alloc<NColour>()
    nc.r = c.r; nc.g = c.g; nc.b = c.b; nc.a = c.a
    return nc
}


internal fun readShapeCastStatus(s: UInt): ShapeCastStatus = when (s) {
    NShapeCastStatusTag_OutOfIterations -> ShapeCastStatus.OutOfIterations
    NShapeCastStatusTag_Converged -> ShapeCastStatus.Converged
    NShapeCastStatusTag_Failed -> ShapeCastStatus.Failed
    NShapeCastStatusTag_PenetratingOrWithinTargetDist -> ShapeCastStatus.PenetratingOrWithinTargetDist
    else -> ShapeCastStatus.Failed
}

internal fun readColliderShape(ffi: ColliderShapeFfi): ColliderShape = when (ffi.tag) {
    ColliderShapeTag_Box -> ColliderShape.Box(
        Vector3d(ffi.data.Box.half_extents.x.toDouble(), ffi.data.Box.half_extents.y.toDouble(), ffi.data.Box.half_extents.z.toDouble())
    )
    ColliderShapeTag_Sphere -> ColliderShape.Sphere(ffi.data.Sphere.radius)
    ColliderShapeTag_Capsule -> ColliderShape.Capsule(ffi.data.Capsule.half_height, ffi.data.Capsule.radius)
    ColliderShapeTag_Cylinder -> ColliderShape.Cylinder(ffi.data.Cylinder.half_height, ffi.data.Cylinder.radius)
    ColliderShapeTag_Cone -> ColliderShape.Cone(ffi.data.Cone.half_height, ffi.data.Cone.radius)
    else -> ColliderShape.Box(Vector3d.zero())
}

internal fun MemScope.allocColliderShape(shape: ColliderShape): ColliderShapeFfi {
    val ffi = alloc<ColliderShapeFfi>()
    when (shape) {
        is ColliderShape.Box -> {
            ffi.tag = ColliderShapeTag_Box
            ffi.data.Box.half_extents.x = shape.halfExtents.x.toFloat()
            ffi.data.Box.half_extents.y = shape.halfExtents.y.toFloat()
            ffi.data.Box.half_extents.z = shape.halfExtents.z.toFloat()
        }
        is ColliderShape.Sphere -> {
            ffi.tag = ColliderShapeTag_Sphere
            ffi.data.Sphere.radius = shape.radius
        }
        is ColliderShape.Capsule -> {
            ffi.tag = ColliderShapeTag_Capsule
            ffi.data.Capsule.half_height = shape.halfHeight
            ffi.data.Capsule.radius = shape.radius
        }
        is ColliderShape.Cylinder -> {
            ffi.tag = ColliderShapeTag_Cylinder
            ffi.data.Cylinder.half_height = shape.halfHeight
            ffi.data.Cylinder.radius = shape.radius
        }
        is ColliderShape.Cone -> {
            ffi.tag = ColliderShapeTag_Cone
            ffi.data.Cone.half_height = shape.halfHeight
            ffi.data.Cone.radius = shape.radius
        }
    }
    return ffi
}
