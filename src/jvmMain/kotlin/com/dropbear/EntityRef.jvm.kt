package com.dropbear

actual fun EntityRef.Companion.getEntityLabel(entity: EntityId): String {
    return EntityRefNative.getEntityLabel(DropbearEngine.native.worldHandle, entity.raw)
        ?: throw RuntimeException("All entities are expected to contain a \"Label\" component. If not, its an engine bug or you messed around with the scene config...")
}

actual fun EntityRef.getChildren(entityId: EntityId): Array<EntityRef>? {
    return EntityRefNative.getChildren(DropbearEngine.native.worldHandle, entityId.raw)
        .map { EntityRef(EntityId(it)) }.toList().toTypedArray()
}

actual fun EntityRef.getChildByLabel(
    entityId: EntityId,
    label: String
): EntityRef? {
    val result = EntityRefNative.getChildByLabel(DropbearEngine.native.worldHandle, entityId.raw, label)
    return if (result != null) {
        EntityRef(EntityId(result))
    } else {
        null
    }
}

actual fun EntityRef.getParent(entityId: EntityId): EntityRef? {
    val result = EntityRefNative.getParent(DropbearEngine.native.worldHandle, entityId.raw)
    return if (result != null) {
        EntityRef(EntityId(result))
    } else {
        null
    }
}