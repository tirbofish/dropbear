@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import kotlin.String
import com.dropbear.math.Vector3d
import kotlinx.cinterop.*

actual fun CustomProperties.getStringProperty(entityHandle: Long, label: String): String? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<CPointerVar<ByteVar>>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_properties_get_string_property(world, entityHandle.toULong(), label, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else out.value?.toKString()
}

actual fun CustomProperties.getIntProperty(entityHandle: Long, label: String): Int? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<IntVar>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_properties_get_int_property(world, entityHandle.toULong(), label, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else out.value
}

actual fun CustomProperties.getLongProperty(entityHandle: Long, label: String): Long? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<LongVar>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_properties_get_long_property(world, entityHandle.toULong(), label, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else out.value
}

actual fun CustomProperties.getDoubleProperty(entityHandle: Long, label: String): Double? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<DoubleVar>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_properties_get_double_property(world, entityHandle.toULong(), label, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else out.value
}

actual fun CustomProperties.getFloatProperty(entityHandle: Long, label: String): Float? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<FloatVar>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_properties_get_float_property(world, entityHandle.toULong(), label, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else out.value
}

actual fun CustomProperties.getBoolProperty(entityHandle: Long, label: String): Boolean? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<BooleanVar>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_properties_get_bool_property(world, entityHandle.toULong(), label, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else out.value
}

actual fun CustomProperties.getVec3Property(entityHandle: Long, label: String): Vector3d? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<NVector3>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_properties_get_vec3_property(world, entityHandle.toULong(), label, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else Vector3d(out.x, out.y, out.z)
}

actual fun CustomProperties.setStringProperty(entityHandle: Long, label: String, value: String) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_properties_set_string_property(world, entityHandle.toULong(), label, value)
}

actual fun CustomProperties.setIntProperty(entityHandle: Long, label: String, value: Int) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_properties_set_int_property(world, entityHandle.toULong(), label, value)
}

actual fun CustomProperties.setLongProperty(entityHandle: Long, label: String, value: Long) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_properties_set_long_property(world, entityHandle.toULong(), label, value)
}

actual fun CustomProperties.setFloatProperty(entityHandle: Long, label: String, value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_properties_set_float_property(world, entityHandle.toULong(), label, value)
}

actual fun CustomProperties.setBoolProperty(entityHandle: Long, label: String, value: Boolean) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_properties_set_bool_property(world, entityHandle.toULong(), label, value)
}

actual fun CustomProperties.setVec3Property(entityHandle: Long, label: String, value: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val nv = allocVec3(value)
    dropbear_properties_set_vec3_property(world, entityHandle.toULong(), label, nv.ptr)
}

internal actual fun customPropertiesExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_properties_custom_properties_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}