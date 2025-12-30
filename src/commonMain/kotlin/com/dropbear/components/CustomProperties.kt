package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3d
import com.dropbear.math.Vector3f
import com.dropbear.math.Vector3i

class CustomProperties(val id: EntityId): Component(id, "CustomProperties") {
    /**
     * Fetches the property of the ModelProperty component on the entity.
     *
     * @param T The type of the property to fetch.
     * # Supported types
     * - [kotlin.String]
     * - [Long]
     * - [Int]
     * - [Double]
     * - [Float]
     * - [Boolean]
     * - [Vector3d]
     * - [Vector3f]
     * - [Vector3i]
     * @param key The key of the property to fetch.
     * @return The property value, or null if the property does not exist.
     * @throws IllegalArgumentException if the property type is not supported.
     */
    inline fun <reified T> getProperty(key: String): T? {
        return when (T::class) {
            String::class -> getStringProperty(id.raw, key) as T?
            Long::class -> getLongProperty(id.raw, key) as T?
            Int::class -> getIntProperty(id.raw, key) as T?
            Double::class -> getDoubleProperty(id.raw, key) as T?

            Float::class -> getFloatProperty(id.raw, key) as T?
            Boolean::class -> getBoolProperty(id.raw, key) as T?
            Vector3d::class -> getVec3Property(id.raw, key) as T?
            Vector3f::class -> getVec3Property(id.raw, key)?.toFloat() as T?
            Vector3i::class -> getVec3Property(id.raw, key)?.toInt() as T?
            else -> throw IllegalArgumentException("Unsupported property type: ${T::class}")
        }
    }

    /**
     * Sets a property of the ModelProperty component on the entity.
     *
     * @param key The key of the property to set.
     * @param value The type of the property to set.
     * # Supported types
     * - [kotlin.String]
     * - [Long]
     * - [Int]
     * - [Double]
     * - [Float]
     * - [Boolean]
     * - [Vector3d]
     * - [Vector3f]
     * - [Vector3i]
     * @throws IllegalArgumentException if the property type is not supported.
     */
    fun setProperty(key: String, value: Any) {
        when (value) {
            is String -> setStringProperty(id.raw, key, value)
            is Long -> setLongProperty(id.raw, key, value)
            is Int -> setIntProperty(id.raw, key, value)
            is Double -> setFloatProperty(id.raw, key, value)
            is Float -> setFloatProperty(id.raw, key, value.toDouble())
            is Boolean -> setBoolProperty(id.raw, key, value)
            is Vector3d -> setVec3Property(id.raw, key, value)
            is Vector3f -> setVec3Property(id.raw ,key, value.toDouble())
            is Vector3i -> setVec3Property(id.raw, key, value.toDouble())
            else -> throw IllegalArgumentException("Unsupported property type: ${value::class}")
        }
    }

    companion object : ComponentType<CustomProperties> {
        override fun get(entityId: EntityId): CustomProperties? {
            return if (customPropertiesExistsForEntity(entityId)) CustomProperties(entityId) else null
        }
    }
}

expect fun CustomProperties.getStringProperty(entityHandle: Long, label: String): String?
expect fun CustomProperties.getIntProperty(entityHandle: Long, label: String): Int?
expect fun CustomProperties.getLongProperty(entityHandle: Long, label: String): Long?
expect fun CustomProperties.getDoubleProperty(entityHandle: Long, label: String): Double?
expect fun CustomProperties.getFloatProperty(entityHandle: Long, label: String): Float?
expect fun CustomProperties.getBoolProperty(entityHandle: Long, label: String): Boolean?
expect fun CustomProperties.getVec3Property(entityHandle: Long, label: String): Vector3d?

expect fun CustomProperties.setStringProperty(entityHandle: Long, label: String, value: String)
expect fun CustomProperties.setIntProperty(entityHandle: Long, label: String, value: Int)
expect fun CustomProperties.setLongProperty(entityHandle: Long, label: String, value: Long)
expect fun CustomProperties.setFloatProperty(entityHandle: Long, label: String, value: Double)
expect fun CustomProperties.setBoolProperty(entityHandle: Long, label: String, value: Boolean)
expect fun CustomProperties.setVec3Property(entityHandle: Long, label: String, value: Vector3d)

expect fun customPropertiesExistsForEntity(entityId: EntityId): Boolean