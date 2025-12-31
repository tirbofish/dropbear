package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.math.Transform

actual fun entityTransformExistsForEntity(entityId: EntityId): Boolean {
    return EntityTransformNative.entityTransformExistsForEntity(DropbearEngine.native.worldHandle, entityId.raw)
}

actual fun EntityTransform.getLocalTransform(entityId: EntityId): Transform {
    return EntityTransformNative.getLocalTransform(DropbearEngine.native.worldHandle, entityId.raw)
        ?: Transform.identity()
}

actual fun EntityTransform.setLocalTransform(
    entityId: EntityId,
    transform: Transform
) {
    EntityTransformNative.setLocalTransform(
        DropbearEngine.native.worldHandle,
        entityId.raw,
        transform
    )
}

actual fun EntityTransform.getWorldTransform(entityId: EntityId): Transform {
    return EntityTransformNative.getWorldTransform(DropbearEngine.native.worldHandle, entityId.raw) ?: Transform.identity()
}

actual fun EntityTransform.setWorldTransform(
    entityId: EntityId,
    transform: Transform
) {
    EntityTransformNative.setWorldTransform(
        DropbearEngine.native.worldHandle,
        entityId.raw,
        transform
    )
}

actual fun EntityTransform.propagateTransform(entityId: EntityId): Transform? {
    return EntityTransformNative.propagateTransform(DropbearEngine.native.worldHandle, entityId.raw)
}