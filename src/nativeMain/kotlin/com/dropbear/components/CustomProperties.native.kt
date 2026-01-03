package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.math.Vector3d

internal actual fun CustomProperties.getStringProperty(
    entityHandle: Long,
    label: String
): String? {
    TODO("Not yet implemented")
}

internal actual fun CustomProperties.getIntProperty(entityHandle: Long, label: String): Int? {
    TODO("Not yet implemented")
}

internal actual fun CustomProperties.getLongProperty(
    entityHandle: Long,
    label: String
): Long? {
    TODO("Not yet implemented")
}

internal actual fun CustomProperties.getDoubleProperty(
    entityHandle: Long,
    label: String
): Double? {
    TODO("Not yet implemented")
}

internal actual fun CustomProperties.getFloatProperty(
    entityHandle: Long,
    label: String
): Float? {
    TODO("Not yet implemented")
}

internal actual fun CustomProperties.getBoolProperty(
    entityHandle: Long,
    label: String
): Boolean? {
    TODO("Not yet implemented")
}

internal actual fun CustomProperties.getVec3Property(
    entityHandle: Long,
    label: String
): Vector3d? {
    TODO("Not yet implemented")
}

internal actual fun CustomProperties.setStringProperty(
    entityHandle: Long,
    label: String,
    value: String
) {
}

internal actual fun CustomProperties.setIntProperty(
    entityHandle: Long,
    label: String,
    value: Int
) {
}

internal actual fun CustomProperties.setLongProperty(
    entityHandle: Long,
    label: String,
    value: Long
) {
}

internal actual fun CustomProperties.setFloatProperty(
    entityHandle: Long,
    label: String,
    value: Double
) {
}

internal actual fun CustomProperties.setBoolProperty(
    entityHandle: Long,
    label: String,
    value: Boolean
) {
}

internal actual fun CustomProperties.setVec3Property(
    entityHandle: Long,
    label: String,
    value: Vector3d
) {
}

internal actual fun customPropertiesExistsForEntity(entityId: EntityId): Boolean {
    TODO("Not yet implemented")
}