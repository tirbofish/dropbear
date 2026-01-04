package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.math.Vector3d

actual fun CustomProperties.getStringProperty(
    entityHandle: Long,
    label: String
): String? {
    return CustomPropertiesNative.getStringProperty(DropbearEngine.native.worldHandle, entityHandle, label)
}

actual fun CustomProperties.getIntProperty(entityHandle: Long, label: String): Int? {
    return CustomPropertiesNative.getIntProperty(DropbearEngine.native.worldHandle, entityHandle, label)
}

actual fun CustomProperties.getLongProperty(
    entityHandle: Long,
    label: String
): Long? {
    return CustomPropertiesNative.getLongProperty(DropbearEngine.native.worldHandle, entityHandle, label)
}

actual fun CustomProperties.getDoubleProperty(
    entityHandle: Long,
    label: String
): Double? {
    return CustomPropertiesNative.getDoubleProperty(DropbearEngine.native.worldHandle, entityHandle, label)
}

actual fun CustomProperties.getFloatProperty(
    entityHandle: Long,
    label: String
): Float? {
    return CustomPropertiesNative.getFloatProperty(DropbearEngine.native.worldHandle, entityHandle, label)
}

actual fun CustomProperties.getBoolProperty(
    entityHandle: Long,
    label: String
): Boolean? {
    return CustomPropertiesNative.getBoolProperty(DropbearEngine.native.worldHandle, entityHandle, label)
}

actual fun CustomProperties.getVec3Property(
    entityHandle: Long,
    label: String
): Vector3d? {
    return CustomPropertiesNative.getVec3Property(DropbearEngine.native.worldHandle, entityHandle, label)
}

actual fun CustomProperties.setStringProperty(
    entityHandle: Long,
    label: String,
    value: String
) {
    CustomPropertiesNative.setStringProperty(DropbearEngine.native.worldHandle, entityHandle, label, value)
}

actual fun CustomProperties.setIntProperty(
    entityHandle: Long,
    label: String,
    value: Int
) {
    CustomPropertiesNative.setIntProperty(DropbearEngine.native.worldHandle, entityHandle, label, value)
}

actual fun CustomProperties.setLongProperty(
    entityHandle: Long,
    label: String,
    value: Long
) {
    CustomPropertiesNative.setLongProperty(DropbearEngine.native.worldHandle, entityHandle, label, value)
}

actual fun CustomProperties.setFloatProperty(
    entityHandle: Long,
    label: String,
    value: Double
) {
    CustomPropertiesNative.setFloatProperty(DropbearEngine.native.worldHandle, entityHandle, label, value)
}

actual fun CustomProperties.setBoolProperty(
    entityHandle: Long,
    label: String,
    value: Boolean
) {
    CustomPropertiesNative.setBoolProperty(DropbearEngine.native.worldHandle, entityHandle, label, value)
}

actual fun CustomProperties.setVec3Property(
    entityHandle: Long,
    label: String,
    value: Vector3d
) {
    CustomPropertiesNative.setVec3Property(DropbearEngine.native.worldHandle, entityHandle, label, value)
}

internal actual fun customPropertiesExistsForEntity(entityId: EntityId): Boolean {
    return CustomPropertiesNative.customPropertiesExistsForEntity(DropbearEngine.native.worldHandle, entityId.raw)
}