@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.components.camera

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import com.dropbear.math.Vector3d
import com.dropbear.math.Vector3f
import kotlinx.cinterop.*

internal actual fun onRailsExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_transform_on_rails_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}

internal actual fun OnRails.onRailsGetEnabled(): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_transform_on_rails_get_enabled(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun OnRails.onRailsSetEnabled(enabled: Boolean) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_transform_on_rails_set_enabled(world, entity.raw.toULong(), enabled)
}

internal actual fun OnRails.onRailsGetProgress(): Float = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0f
    val out = alloc<FloatVar>()
    dropbear_transform_on_rails_get_progress(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun OnRails.onRailsSetProgress(progress: Float) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_transform_on_rails_set_progress(world, entity.raw.toULong(), progress)
}

internal actual fun OnRails.onRailsGetPath(): List<Vector3d> = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped emptyList()
    val lenOut = alloc<IntVar>()
    val rc = dropbear_transform_on_rails_get_path_len(world, entity.raw.toULong(), lenOut.ptr)
    if (rc != 0) return@memScoped emptyList()
    val len = lenOut.value
    (0 until len).mapNotNull { i ->
        val out = alloc<NVector3>()
        val rc2 = dropbear_transform_on_rails_get_path_point(world, entity.raw.toULong(), i, out.ptr)
        if (rc2 != 0) null else Vector3d(out.x, out.y, out.z)
    }
}

internal actual fun OnRails.onRailsSetPath(path: List<Vector3d>) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_transform_on_rails_clear_path(world, entity.raw.toULong())
    for (point in path) {
        val v = alloc<NVector3>()
        v.x = point.x; v.y = point.y; v.z = point.z
        dropbear_transform_on_rails_push_path_point(world, entity.raw.toULong(), v.ptr)
    }
}

internal actual fun OnRails.onRailsGetDrive(): RailDrive = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped RailDrive.Manual
    val typeOut = alloc<IntVar>()
    val rc = dropbear_transform_on_rails_get_drive_type(world, entity.raw.toULong(), typeOut.ptr)
    if (rc != 0) return@memScoped RailDrive.Manual

    when (typeOut.value) {
        0 -> {
            val speed = alloc<FloatVar>()
            val looping = alloc<BooleanVar>()
            dropbear_transform_on_rails_get_drive_automatic_speed(world, entity.raw.toULong(), speed.ptr)
            dropbear_transform_on_rails_get_drive_automatic_looping(world, entity.raw.toULong(), looping.ptr)
            RailDrive.Automatic(speed.value, looping.value)
        }
        1 -> {
            val target = alloc<ULongVar>()
            val monotonic = alloc<BooleanVar>()
            dropbear_transform_on_rails_get_drive_follow_entity_target(world, entity.raw.toULong(), target.ptr)
            dropbear_transform_on_rails_get_drive_follow_entity_monotonic(world, entity.raw.toULong(), monotonic.ptr)
            RailDrive.FollowEntity(EntityId(target.value.toLong()), monotonic.value)
        }
        2 -> {
            val target = alloc<ULongVar>()
            val axis = alloc<NVector3>()
            val rangeMin = alloc<FloatVar>()
            val rangeMax = alloc<FloatVar>()
            dropbear_transform_on_rails_get_drive_axis_driven_target(world, entity.raw.toULong(), target.ptr)
            dropbear_transform_on_rails_get_drive_axis_driven_axis(world, entity.raw.toULong(), axis.ptr)
            dropbear_transform_on_rails_get_drive_axis_driven_range_min(world, entity.raw.toULong(), rangeMin.ptr)
            dropbear_transform_on_rails_get_drive_axis_driven_range_max(world, entity.raw.toULong(), rangeMax.ptr)
            RailDrive.AxisDriven(
                EntityId(target.value.toLong()),
                Vector3f(axis.x.toFloat(), axis.y.toFloat(), axis.z.toFloat()),
                rangeMin.value,
                rangeMax.value,
            )
        }
        else -> RailDrive.Manual
    }
}

internal actual fun OnRails.onRailsSetDrive(drive: RailDrive) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val entityBits = entity.raw.toULong()
    when (drive) {
        is RailDrive.Automatic ->
            dropbear_transform_on_rails_set_drive_automatic(world, entityBits, drive.speed, drive.looping)
        is RailDrive.FollowEntity ->
            dropbear_transform_on_rails_set_drive_follow_entity(world, entityBits, drive.target.raw.toULong(), drive.monotonic)
        is RailDrive.AxisDriven -> {
            val v = alloc<NVector3>()
            v.x = drive.axis.x.toDouble(); v.y = drive.axis.y.toDouble(); v.z = drive.axis.z.toDouble()
            dropbear_transform_on_rails_set_drive_axis_driven(world, entityBits, drive.target.raw.toULong(), v.ptr, drive.rangeMin, drive.rangeMax)
        }
        is RailDrive.Manual ->
            dropbear_transform_on_rails_set_drive_manual(world, entityBits)
    }
}
