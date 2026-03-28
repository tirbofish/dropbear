package com.dropbear.components.camera

import com.dropbear.EntityId
import com.dropbear.ecs.ComponentType
import com.dropbear.ecs.ExternalComponent
import com.dropbear.math.Point
import com.dropbear.math.Vector3f

/**
 * Controls how the [OnRails] component advances its progress along the path.
 */
sealed class RailDrive {
    /**
     * Progress advances automatically at a fixed speed.
     *
     * Use for cutscenes, intros, or automated camera pans.
     *
     * @param speed Units of progress (0.0–1.0) advanced per second.
     * @param looping When `true`, progress wraps back to 0.0 at the end.
     */
    data class Automatic(
        val speed: Float,
        val looping: Boolean,
    ) : RailDrive()

    /**
     * Progress is tied to the closest point on the rail to a target entity.
     *
     * Use for third-person or side-scroller cameras following a player.
     *
     * @param target The entity whose position drives rail progress.
     * @param monotonic When `true`, progress can never decrease. The camera won't scroll backward.
     */
    data class FollowEntity(
        val target: EntityId,
        val monotonic: Boolean = false,
    ) : RailDrive()

    /**
     * Progress is driven by a specific axis of the target entity's world position.
     *
     * e.g. `AxisDriven(player, Vector3f(1f, 0f, 0f), 0f, 100f)` scrolls as the player moves
     * along the X axis from 0 to 100 world units.
     *
     * Use for side-scrollers or top-down cameras with a dominant movement axis.
     *
     * @param target The entity whose position is projected onto [axis].
     * @param axis World-space direction vector to project onto.
     * @param rangeMin World-space axis value that maps to progress 0.0.
     * @param rangeMax World-space axis value that maps to progress 1.0.
     */
    data class AxisDriven(
        val target: EntityId,
        val axis: Vector3f,
        val rangeMin: Float,
        val rangeMax: Float,
    ) : RailDrive()

    /**
     * Progress is set entirely by external code (scripting, cutscene sequencer, etc.).
     *
     * The system will not touch [OnRails.progress] at all.
     */
    data object Manual : RailDrive()
}

/**
 * Constrains an entity's movement to a fixed spline path.
 *
 * The entity must also have an `EntityTransform` component. [progress] is a value in `[0.0, 1.0]`
 * that indicates how far along the path the entity sits; the native system updates `world.position`
 * every frame based on the active [drive] mode.
 *
 * @param entity The entity this component is attached to.
 */
class OnRails(
    val entity: EntityId,
) : ExternalComponent("eucalyptus_core::transform::OnRails") {

    /**
     * Whether the component actively moves the entity along the path each frame.
     *
     * When `false`, the system will not update the entity's position from the path; `progress`
     * can still be read or written externally (e.g., for cutscene sequencers).
     */
    var enabled: Boolean
        get() = onRailsGetEnabled()
        set(value) = onRailsSetEnabled(value)

    /**
     * The ordered list of world-space waypoints that define the rail.
     *
     * Requires at least 2 points. Each [com.dropbear.math.Point] carries a position and an optional explicit
     * rotation. When [com.dropbear.math.Point.rotation] is `null`, orientation is derived from the path tangent.
     */
    var path: List<Point>
        get() = onRailsGetPath()
        set(value) = onRailsSetPath(value)

    /**
     * Current interpolation position along the rail, in the range `[0.0, 1.0]`.
     *
     * `0.0` is the first waypoint and `1.0` is the last. Can be set directly when using
     * [RailDrive.Manual].
     */
    var progress: Float
        get() = onRailsGetProgress()
        set(value) = onRailsSetProgress(value)

    /**
     * How the rail progress is updated each frame.
     *
     * @see RailDrive
     */
    var drive: RailDrive
        get() = onRailsGetDrive()
        set(value) = onRailsSetDrive(value)

    companion object : ComponentType<OnRails> {
        override fun get(entityId: EntityId): OnRails? {
            return if (onRailsExistsForEntity(entityId)) OnRails(entityId) else null
        }
    }
}

internal expect fun onRailsExistsForEntity(entityId: EntityId): Boolean

internal expect fun OnRails.onRailsGetEnabled(): Boolean
internal expect fun OnRails.onRailsSetEnabled(enabled: Boolean)

internal expect fun OnRails.onRailsGetPath(): List<Point>
internal expect fun OnRails.onRailsSetPath(path: List<Point>)

internal expect fun OnRails.onRailsGetProgress(): Float
internal expect fun OnRails.onRailsSetProgress(progress: Float)

internal expect fun OnRails.onRailsGetDrive(): RailDrive
internal expect fun OnRails.onRailsSetDrive(drive: RailDrive)
