package com.dropbear.components.camera

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.math.Vector3d
import com.dropbear.math.Vector3f

internal actual fun onRailsExistsForEntity(entityId: EntityId): Boolean =
    OnRailsNative.existsForEntity(DropbearEngine.native.worldHandle, entityId.raw)

internal actual fun OnRails.onRailsGetEnabled(): Boolean =
    OnRailsNative.getEnabled(DropbearEngine.native.worldHandle, entity.raw)

internal actual fun OnRails.onRailsSetEnabled(enabled: Boolean) =
    OnRailsNative.setEnabled(DropbearEngine.native.worldHandle, entity.raw, enabled)

internal actual fun OnRails.onRailsGetProgress(): Float =
    OnRailsNative.getProgress(DropbearEngine.native.worldHandle, entity.raw)

internal actual fun OnRails.onRailsSetProgress(progress: Float) =
    OnRailsNative.setProgress(DropbearEngine.native.worldHandle, entity.raw, progress)

internal actual fun OnRails.onRailsGetPath(): List<Vector3d> {
    val world = DropbearEngine.native.worldHandle
    val len = OnRailsNative.getPathLen(world, entity.raw)
    return (0 until len).map { i -> OnRailsNative.getPathPoint(world, entity.raw, i) }
}

internal actual fun OnRails.onRailsSetPath(path: List<Vector3d>) {
    val world = DropbearEngine.native.worldHandle
    OnRailsNative.clearPath(world, entity.raw)
    path.forEach { p -> OnRailsNative.pushPathPoint(world, entity.raw, p) }
}

internal actual fun OnRails.onRailsGetDrive(): RailDrive {
    val world = DropbearEngine.native.worldHandle
    val entityRaw = entity.raw
    return when (OnRailsNative.getDriveType(world, entityRaw)) {
        0 -> RailDrive.Automatic(
            speed   = OnRailsNative.getDriveAutomaticSpeed(world, entityRaw),
            looping = OnRailsNative.getDriveAutomaticLooping(world, entityRaw),
        )
        1 -> RailDrive.FollowEntity(
            target    = EntityId(OnRailsNative.getDriveFollowEntityTarget(world, entityRaw)),
            monotonic = OnRailsNative.getDriveFollowEntityMonotonic(world, entityRaw),
        )
        2 -> {
            val axisD = OnRailsNative.getDriveAxisDrivenAxis(world, entityRaw)
            RailDrive.AxisDriven(
                target   = EntityId(OnRailsNative.getDriveAxisDrivenTarget(world, entityRaw)),
                axis     = Vector3f(axisD.x.toFloat(), axisD.y.toFloat(), axisD.z.toFloat()),
                rangeMin = OnRailsNative.getDriveAxisDrivenRangeMin(world, entityRaw),
                rangeMax = OnRailsNative.getDriveAxisDrivenRangeMax(world, entityRaw),
            )
        }
        else -> RailDrive.Manual
    }
}

internal actual fun OnRails.onRailsSetDrive(drive: RailDrive) {
    val world = DropbearEngine.native.worldHandle
    val entityRaw = entity.raw
    when (drive) {
        is RailDrive.Automatic ->
            OnRailsNative.setDriveAutomatic(world, entityRaw, drive.speed, drive.looping)
        is RailDrive.FollowEntity ->
            OnRailsNative.setDriveFollowEntity(world, entityRaw, drive.target.raw, drive.monotonic)
        is RailDrive.AxisDriven ->
            OnRailsNative.setDriveAxisDriven(
                world, entityRaw, drive.target.raw,
                Vector3d(drive.axis.x.toDouble(), drive.axis.y.toDouble(), drive.axis.z.toDouble()),
                drive.rangeMin, drive.rangeMax,
            )
        is RailDrive.Manual ->
            OnRailsNative.setDriveManual(world, entityRaw)
    }
}
