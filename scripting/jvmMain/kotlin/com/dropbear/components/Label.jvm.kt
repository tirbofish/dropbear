package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId

internal actual fun labelExistsForEntity(entityId: EntityId): Boolean {
    return LabelNative.labelExistsForEntity(DropbearEngine.native.worldHandle, entityId.raw)
}