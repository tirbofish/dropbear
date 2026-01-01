package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityId

actual fun ColliderGroup.getColliderGroupColliders(colliderGroup: ColliderGroup): List<Collider> {
    return ColliderGroupNative.getColliderGroupColliders(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, colliderGroup.entity.raw).toList()
}

actual fun colliderGroupExistsForEntity(entityId: EntityId): Boolean {
    return ColliderGroupNative.colliderGroupExistsForEntity(DropbearEngine.native.worldHandle, entityId.raw)
}