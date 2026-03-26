package com.dropbear.rendering

import com.dropbear.DropbearEngine
import com.dropbear.math.Quaterniond
import com.dropbear.math.Vector3d
import com.dropbear.utils.Colour

private val g get() = DropbearEngine.native.graphicsContextHandle

internal actual fun DebugDraw.drawLineNative(start: Vector3d, end: Vector3d, colour: Colour) =
    DebugDrawNative.drawLine(g, start, end, colour)

internal actual fun DebugDraw.drawRayNative(origin: Vector3d, dir: Vector3d, colour: Colour) =
    DebugDrawNative.drawRay(g, origin, dir, colour)

internal actual fun DebugDraw.drawArrowNative(start: Vector3d, end: Vector3d, colour: Colour) =
    DebugDrawNative.drawArrow(g, start, end, colour)

internal actual fun DebugDraw.drawPointNative(pos: Vector3d, size: Float, colour: Colour) =
    DebugDrawNative.drawPoint(g, pos, size, colour)

internal actual fun DebugDraw.drawCircleNative(center: Vector3d, radius: Float, normal: Vector3d, colour: Colour) =
    DebugDrawNative.drawCircle(g, center, radius, normal, colour)

internal actual fun DebugDraw.drawSphereNative(center: Vector3d, radius: Float, colour: Colour) =
    DebugDrawNative.drawSphere(g, center, radius, colour)

internal actual fun DebugDraw.drawGlobeNative(center: Vector3d, radius: Float, latLines: Int, lonLines: Int, colour: Colour) =
    DebugDrawNative.drawGlobe(g, center, radius, latLines, lonLines, colour)

internal actual fun DebugDraw.drawAabbNative(min: Vector3d, max: Vector3d, colour: Colour) =
    DebugDrawNative.drawAabb(g, min, max, colour)

internal actual fun DebugDraw.drawObbNative(center: Vector3d, halfExtents: Vector3d, rotation: Quaterniond, colour: Colour) =
    DebugDrawNative.drawObb(g, center, halfExtents, rotation, colour)

internal actual fun DebugDraw.drawCapsuleNative(a: Vector3d, b: Vector3d, radius: Float, colour: Colour) =
    DebugDrawNative.drawCapsule(g, a, b, radius, colour)

internal actual fun DebugDraw.drawConeNative(apex: Vector3d, dir: Vector3d, angle: Float, length: Float, colour: Colour) =
    DebugDrawNative.drawCone(g, apex, dir, angle, length, colour)
