@file:OptIn(ExperimentalForeignApi::class, ExperimentalNativeApi::class)
@file:Suppress("EXPECT_ACTUAL_CLASSIFIERS_ARE_IN_BETA_WARNING")

package com.dropbear.ffi

import com.dropbear.EntityId
import com.dropbear.exception.DropbearNativeException
import com.dropbear.ffi.generated.ColliderShape
import com.dropbear.ffi.generated.ColliderShape_Box
import com.dropbear.ffi.generated.ColliderShape_Capsule
import com.dropbear.ffi.generated.ColliderShape_Cone
import com.dropbear.ffi.generated.ColliderShape_Cylinder
import com.dropbear.ffi.generated.ColliderShape_Sphere
import com.dropbear.ffi.generated.RigidBodyMode
import com.dropbear.ffi.generated.SceneLoadResult
import com.dropbear.ffi.generated.Vector3D
import com.dropbear.physics.AxisLock
import com.dropbear.physics.Collider
import com.dropbear.physics.Index
import com.dropbear.physics.RigidBody
import com.dropbear.scene.SceneLoadStatus
import kotlinx.cinterop.ExperimentalForeignApi
import kotlin.experimental.ExperimentalNativeApi

internal fun SceneLoadResult.fromNative(): SceneLoadStatus {
    return when (this) {
        SceneLoadResult.SCENE_LOAD_PENDING -> SceneLoadStatus.PENDING
        SceneLoadResult.SCENE_LOAD_SUCCESS -> SceneLoadStatus.READY
        SceneLoadResult.SCENE_LOAD_ERROR -> SceneLoadStatus.FAILED
    }
}

internal fun Vector3D.toKotlin(): com.dropbear.math.Vector3D {
    return com.dropbear.math.Vector3D(this.x, this.y, this.z)
}

internal fun RigidBodyMode.toKotlin(): com.dropbear.physics.RigidBodyMode {
    return when (this) {
        RigidBodyMode.RIGIDBODY_MODE_DYNAMIC -> com.dropbear.physics.RigidBodyMode.Dynamic
        RigidBodyMode.RIGIDBODY_MODE_FIXED -> com.dropbear.physics.RigidBodyMode.Fixed
        RigidBodyMode.RIGIDBODY_MODE_KINEMATIC_POSITION -> com.dropbear.physics.RigidBodyMode.KinematicPosition
        RigidBodyMode.RIGIDBODY_MODE_KINEMATIC_VELOCITY -> com.dropbear.physics.RigidBodyMode.KinematicVelocity
    }
}

internal fun com.dropbear.ffi.generated.AxisLock.toKotlin(): AxisLock {
    return AxisLock(
        this.x,
        this.y,
        this.z,
    )
}

internal fun com.dropbear.ffi.generated.RigidBody.toKotlin(nativeEngine: NativeEngine): RigidBody {
    return RigidBody(
        index = Index(this.index.index, this.index.generation),
        entity = EntityId(this.entity),
        rigidBodyMode = this.mode.toKotlin(),
        gravityScale = this.gravity_scale,
        canSleep = this.can_sleep,
        ccdEnabled = this.ccd_enabled,
        linearVelocity = this.linear_velocity.toKotlin(),
        angularVelocity = this.angualar_velocity.toKotlin(),
        linearDamping = this.linear_damping,
        angularDamping = this.angular_damping,
        lockTranslation = this.lock_translation.toKotlin(),
        lockRotation = this.lock_rotation.toKotlin(),
        native = nativeEngine
    )
}

internal fun RigidBody.populateCStruct(cBody: com.dropbear.ffi.generated.RigidBody) {
    cBody.index.index = this.index.index
    cBody.index.generation = this.index.generation
    cBody.entity = this.entity.id
    cBody.mode = when (this.rigidBodyMode) {
        com.dropbear.physics.RigidBodyMode.Dynamic -> RigidBodyMode.RIGIDBODY_MODE_DYNAMIC
        com.dropbear.physics.RigidBodyMode.Fixed -> RigidBodyMode.RIGIDBODY_MODE_FIXED
        com.dropbear.physics.RigidBodyMode.KinematicPosition -> RigidBodyMode.RIGIDBODY_MODE_KINEMATIC_POSITION
        com.dropbear.physics.RigidBodyMode.KinematicVelocity -> RigidBodyMode.RIGIDBODY_MODE_KINEMATIC_VELOCITY
    }
    cBody.gravity_scale = this.gravityScale
    cBody.can_sleep = this.canSleep
    cBody.ccd_enabled = this.ccdEnabled

    cBody.linear_velocity.x = this.linearVelocity.x
    cBody.linear_velocity.y = this.linearVelocity.y
    cBody.linear_velocity.z = this.linearVelocity.z

    cBody.angualar_velocity.x = this.angularVelocity.x
    cBody.angualar_velocity.y = this.angularVelocity.y
    cBody.angualar_velocity.z = this.angularVelocity.z

    cBody.linear_damping = this.linearDamping
    cBody.angular_damping = this.angularDamping

    cBody.lock_translation.x = this.lockTranslation.x
    cBody.lock_translation.y = this.lockTranslation.y
    cBody.lock_translation.z = this.lockTranslation.z

    cBody.lock_rotation.x = this.lockRotation.x
    cBody.lock_rotation.y = this.lockRotation.y
    cBody.lock_rotation.z = this.lockRotation.z
}

internal fun ColliderShape.toKotlin(): com.dropbear.physics.ColliderShape {
    return when (this.tag) {
        ColliderShape_Box -> {
            com.dropbear.physics.ColliderShape.Box(
                halfExtents = com.dropbear.math.Vector3D(
                    this.data.box.half_extents.x,
                    this.data.box.half_extents.y,
                    this.data.box.half_extents.z
                )
            )
        }
        ColliderShape_Sphere -> {
            com.dropbear.physics.ColliderShape.Sphere(this.data.sphere.radius)
        }
        ColliderShape_Capsule -> {
            com.dropbear.physics.ColliderShape.Capsule(this.data.capsule.half_height, this.data.capsule.radius)
        }
        ColliderShape_Cylinder -> {
            com.dropbear.physics.ColliderShape.Cylinder(this.data.cylinder.half_height, this.data.cylinder.radius)
        }
        ColliderShape_Cone -> {
            com.dropbear.physics.ColliderShape.Cone(this.data.cone.half_height, this.data.cone.radius)
        }
        else -> throw DropbearNativeException("Unknown collider tag: ${this.tag}")
    }
}

internal fun com.dropbear.ffi.generated.Collider.toKotlin(nativeEngine: NativeEngine): Collider {
    return Collider(
        index = Index(this.index.index, this.index.generation),
        entity = EntityId(this.entity),
        colliderShape = this.collider_shape.toKotlin(),
        density = this.density,
        friction = this.friction,
        restitution = this.restitution,
        isSensor = this.is_sensor,
        translation = this.translation.toKotlin(),
        rotation = this.rotation.toKotlin(),
        native = nativeEngine,
        id = this.id,
    )
}

internal fun Collider.populateCStruct(struct: com.dropbear.ffi.generated.Collider) {
    struct.index.index = this.index.index
    struct.index.generation = this.index.generation
    struct.entity = this.entity.id
    struct.density = this.density
    struct.friction = this.friction
    struct.restitution = this.restitution
    struct.is_sensor = this.isSensor

    struct.translation.x = this.translation.x
    struct.translation.y = this.translation.y
    struct.translation.z = this.translation.z

    struct.rotation.x = this.rotation.x
    struct.rotation.y = this.rotation.y
    struct.rotation.z = this.rotation.z

    this.colliderShape.populateCStruct(struct.collider_shape)
}

internal fun com.dropbear.physics.ColliderShape.populateCStruct(struct: ColliderShape) {
    when (this) {
        is com.dropbear.physics.ColliderShape.Box -> {
            struct.tag = ColliderShape_Box
            struct.data.box.half_extents.x = this.halfExtents.x
            struct.data.box.half_extents.y = this.halfExtents.y
            struct.data.box.half_extents.z = this.halfExtents.z
        }
        is com.dropbear.physics.ColliderShape.Sphere -> {
            struct.tag = ColliderShape_Sphere
            struct.data.sphere.radius = this.radius
        }
        is com.dropbear.physics.ColliderShape.Capsule -> {
            struct.tag = ColliderShape_Capsule
            struct.data.capsule.half_height = this.halfHeight
            struct.data.capsule.radius = this.radius
        }
        is com.dropbear.physics.ColliderShape.Cylinder -> {
            struct.tag = ColliderShape_Cylinder
            struct.data.cylinder.half_height = this.halfHeight
            struct.data.cylinder.radius = this.radius
        }
        is com.dropbear.physics.ColliderShape.Cone -> {
            struct.tag = ColliderShape_Cone
            struct.data.cone.half_height = this.halfHeight
            struct.data.cone.radius = this.radius
        }
    }
}