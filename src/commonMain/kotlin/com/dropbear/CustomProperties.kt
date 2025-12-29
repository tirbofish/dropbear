package com.dropbear

import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3

class CustomProperties(val id: EntityId): Component(id, "CustomProperties") {
    /**
     * Fetches the property of the ModelProperty component on the entity.
     */
    inline fun <reified T> getProperty(key: String): T? {
        return when (T::class) {
            String::class -> getStringProperty(id.id, key) as T?
            Long::class -> getLongProperty(id.id, key) as T?
            Int::class -> getIntProperty(id.id, key) as T?
            Double::class -> getDoubleProperty(id.id, key) as T?

            Float::class -> getFloatProperty(id.id, key) as T?
            Boolean::class -> getBoolProperty(id.id, key) as T?
            FloatArray::class -> getVec3Property(id.id, key) as T?
            else -> throw IllegalArgumentException("Unsupported property type: ${T::class}")
        }
    }

    /**
     * Sets a property of the ModelProperty component on the entity.
     *
     * # Supported types
     * - [kotlin.String]
     * - [kotlin.Long]
     * - [kotlin.Int]
     * - [kotlin.Double]
     * - [kotlin.Float]
     * - [kotlin.Boolean]
     * - [com.dropbear.math.Vector3]
     */
    /**
     * Sets a property of the ModelProperty component on the entity.
     *
     * # Supported types
     * - [kotlin.String]
     * - [kotlin.Long]
     * - [kotlin.Int]
     * - [kotlin.Double]
     * - [kotlin.Float]
     * - [kotlin.Boolean]
     * - [com.dropbear.math.Vector3]
     */
    fun setProperty(key: String, value: Any) {
        when (value) {
            is String -> setStringProperty(id.id, key, value)
            is Long -> setLongProperty(id.id, key, value)
            is Int -> setIntProperty(id.id, key, value)
            is Double -> setFloatProperty(id.id, key, value)
            is Float -> setFloatProperty(id.id, key, value.toDouble())
            is Boolean -> setBoolProperty(id.id, key, value)
            is Vector3<*> -> {
                val vec = value.asDoubleVector()
                setVec3Property(id.id, key, floatArrayOf(vec.x.toFloat(), vec.y.toFloat(),
                    vec.z.toFloat()
                ))
            }
            is FloatArray -> {
                require(value.size == 3) { "Vec3 property must have exactly 3 elements" }
                setVec3Property(id.id, key, value)
            }
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
expect fun CustomProperties.getVec3Property(entityHandle: Long, label: String): FloatArray?

expect fun CustomProperties.setStringProperty(entityHandle: Long, label: String, value: String)
expect fun CustomProperties.setIntProperty(entityHandle: Long, label: String, value: Int)
expect fun CustomProperties.setLongProperty(entityHandle: Long, label: String, value: Long)
expect fun CustomProperties.setFloatProperty(entityHandle: Long, label: String, value: Double)
expect fun CustomProperties.setBoolProperty(entityHandle: Long, label: String, value: Boolean)
expect fun CustomProperties.setVec3Property(entityHandle: Long, label: String, value: FloatArray)

expect fun customPropertiesExistsForEntity(entityId: EntityId): Boolean