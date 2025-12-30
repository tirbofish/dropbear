package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.math.Transform

actual fun EntityTransform.getLocalTransform(entityId: EntityId): Transform {
    TODO("Not yet implemented")
}

actual fun EntityTransform.setLocalTransform(
    entityId: EntityId,
    transform: Transform
) {
}

actual fun EntityTransform.getWorldTransform(entityId: EntityId): Transform {
    TODO("Not yet implemented")
}

actual fun EntityTransform.setWorldTransform(
    entityId: EntityId,
    transform: Transform
) {
}

actual fun EntityTransform.propagateTransform(entityId: EntityId): Transform {
    TODO("Not yet implemented")
}

actual fun entityTransformExistsForEntity(entityId: EntityId): Boolean {
    TODO("Not yet implemented")
}