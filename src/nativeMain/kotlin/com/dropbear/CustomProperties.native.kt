package com.dropbear

actual fun CustomProperties.getStringProperty(
    entityHandle: Long,
    label: String
): String? {
    TODO("Not yet implemented")
}

actual fun CustomProperties.getIntProperty(entityHandle: Long, label: String): Int? {
    TODO("Not yet implemented")
}

actual fun CustomProperties.getLongProperty(
    entityHandle: Long,
    label: String
): Long? {
    TODO("Not yet implemented")
}

actual fun CustomProperties.getDoubleProperty(
    entityHandle: Long,
    label: String
): Double? {
    TODO("Not yet implemented")
}

actual fun CustomProperties.getFloatProperty(
    entityHandle: Long,
    label: String
): Float? {
    TODO("Not yet implemented")
}

actual fun CustomProperties.getBoolProperty(
    entityHandle: Long,
    label: String
): Boolean? {
    TODO("Not yet implemented")
}

actual fun CustomProperties.getVec3Property(
    entityHandle: Long,
    label: String
): FloatArray? {
    TODO("Not yet implemented")
}

actual fun CustomProperties.setStringProperty(
    entityHandle: Long,
    label: String,
    value: String
) {
}

actual fun CustomProperties.setIntProperty(
    entityHandle: Long,
    label: String,
    value: Int
) {
}

actual fun CustomProperties.setLongProperty(
    entityHandle: Long,
    label: String,
    value: Long
) {
}

actual fun CustomProperties.setFloatProperty(
    entityHandle: Long,
    label: String,
    value: Double
) {
}

actual fun CustomProperties.setBoolProperty(
    entityHandle: Long,
    label: String,
    value: Boolean
) {
}

actual fun CustomProperties.setVec3Property(
    entityHandle: Long,
    label: String,
    value: FloatArray
) {
}

actual fun customPropertiesExistsForEntity(entityId: EntityId): Boolean {
    TODO("Not yet implemented")
}