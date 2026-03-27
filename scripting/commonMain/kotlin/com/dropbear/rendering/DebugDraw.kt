package com.dropbear.rendering

import com.dropbear.math.Quaterniond
import com.dropbear.math.Vector3d
import com.dropbear.utils.Colour

/**
 * Functions related to drawing to aid with debugging, such as wireframe lines and shapes.
 *
 * All draws are cleared at the end of each frame automatically.
 */
object DebugDraw {
    /** Draws a line between [start] and [end] with the given [colour]. */
    fun drawLine(start: Vector3d, end: Vector3d, colour: Colour = Colour.WHITE) =
        drawLineNative(start, end, colour)

    /** Draws a ray from [origin] in the direction [dir] with the given [colour]. */
    fun drawRay(origin: Vector3d, dir: Vector3d, colour: Colour = Colour.WHITE) =
        drawRayNative(origin, dir, colour)

    /** Draws a line from [start] to [end] with an arrowhead at [end]. */
    fun drawArrow(start: Vector3d, end: Vector3d, colour: Colour = Colour.WHITE) =
        drawArrowNative(start, end, colour)

    /** Draws a cross at [pos] with the given [size]. */
    fun drawPoint(pos: Vector3d, size: Double = 0.1, colour: Colour = Colour.WHITE) =
        drawPointNative(pos, size.toFloat(), colour)

    /**
     * Draws a circle at [center] with the given [radius].
     * [normal] defines the axis the circle faces (e.g. [Vector3d.up] for a flat ground circle).
     */
    fun drawCircle(center: Vector3d, radius: Double, normal: Vector3d = Vector3d.up(), colour: Colour = Colour.WHITE) =
        drawCircleNative(center, radius.toFloat(), normal, colour)

    /**
     * Draws three axis-aligned circles at [center] to approximate a sphere outline.
     * For a proper globe, see [drawGlobe].
     */
    fun drawSphere(center: Vector3d, radius: Double, colour: Colour = Colour.WHITE) =
        drawSphereNative(center, radius.toFloat(), colour)

    /**
     * Draws a wireframe globe at [center] using latitude and longitude rings.
     * [latLines] controls horizontal rings; [lonLines] controls vertical rings.
     */
    fun drawGlobe(center: Vector3d, radius: Double, latLines: Int = 8, lonLines: Int = 8, colour: Colour = Colour.WHITE) =
        drawGlobeNative(center, radius.toFloat(), latLines, lonLines, colour)

    /** Draws a wireframe axis-aligned bounding box (AABB) from [min] to [max]. */
    fun drawAabb(min: Vector3d, max: Vector3d, colour: Colour = Colour.WHITE) =
        drawAabbNative(min, max, colour)

    /**
     * Draws a wireframe oriented bounding box (OBB) at [center].
     * [halfExtents] defines the local half-sizes along each axis.
     * [rotation] orients the box in world space.
     */
    fun drawObb(center: Vector3d, halfExtents: Vector3d, rotation: Quaterniond = Quaterniond.identity(), colour: Colour = Colour.WHITE) =
        drawObbNative(center, halfExtents, rotation, colour)

    /**
     * Draws a wireframe capsule between [a] (bottom) and [b] (top) with the given [radius].
     */
    fun drawCapsule(a: Vector3d, b: Vector3d, radius: Double, colour: Colour = Colour.WHITE) =
        drawCapsuleNative(a, b, radius.toFloat(), colour)

    /**
     * Draws a wireframe cylinder centered at [center], aligned to [axis].
     * [halfHeight] is the half-height along the axis.
     */
    fun drawCylinder(center: Vector3d, halfHeight: Double, radius: Double, axis: Vector3d = Vector3d.up(), colour: Colour = Colour.WHITE) =
        drawCylinderNative(center, halfHeight.toFloat(), radius.toFloat(), axis, colour)

    /**
     * Draws a wireframe cone from [apex] extending in [dir].
     * [angle] is the half-angle in radians. [length] controls how far the cone extends.
     */
    fun drawCone(apex: Vector3d, dir: Vector3d, angle: Double, length: Double, colour: Colour = Colour.WHITE) =
        drawConeNative(apex, dir, angle.toFloat(), length.toFloat(), colour)
}

internal expect fun DebugDraw.drawLineNative(start: Vector3d, end: Vector3d, colour: Colour)
internal expect fun DebugDraw.drawRayNative(origin: Vector3d, dir: Vector3d, colour: Colour)
internal expect fun DebugDraw.drawArrowNative(start: Vector3d, end: Vector3d, colour: Colour)
internal expect fun DebugDraw.drawPointNative(pos: Vector3d, size: Float, colour: Colour)
internal expect fun DebugDraw.drawCircleNative(center: Vector3d, radius: Float, normal: Vector3d, colour: Colour)
internal expect fun DebugDraw.drawSphereNative(center: Vector3d, radius: Float, colour: Colour)
internal expect fun DebugDraw.drawGlobeNative(center: Vector3d, radius: Float, latLines: Int, lonLines: Int, colour: Colour)
internal expect fun DebugDraw.drawAabbNative(min: Vector3d, max: Vector3d, colour: Colour)
internal expect fun DebugDraw.drawObbNative(center: Vector3d, halfExtents: Vector3d, rotation: Quaterniond, colour: Colour)
internal expect fun DebugDraw.drawCapsuleNative(a: Vector3d, b: Vector3d, radius: Float, colour: Colour)
internal expect fun DebugDraw.drawCylinderNative(center: Vector3d, halfHeight: Float, radius: Float, axis: Vector3d, colour: Colour)
internal expect fun DebugDraw.drawConeNative(apex: Vector3d, dir: Vector3d, angle: Float, length: Float, colour: Colour)
