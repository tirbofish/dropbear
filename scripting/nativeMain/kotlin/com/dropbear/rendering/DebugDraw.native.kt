@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.rendering

import com.dropbear.DropbearEngine
import com.dropbear.ffi.generated.*
import com.dropbear.math.Quaterniond
import com.dropbear.math.Vector3d
import com.dropbear.utils.Colour
import kotlinx.cinterop.*

internal actual fun DebugDraw.drawLineNative(start: Vector3d, end: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_line(g, allocVec3(start).ptr, allocVec3(end).ptr, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawRayNative(origin: Vector3d, dir: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_ray(g, allocVec3(origin).ptr, allocVec3(dir).ptr, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawArrowNative(start: Vector3d, end: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_arrow(g, allocVec3(start).ptr, allocVec3(end).ptr, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawPointNative(pos: Vector3d, size: Float, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_point(g, allocVec3(pos).ptr, size, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawCircleNative(center: Vector3d, radius: Float, normal: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_circle(g, allocVec3(center).ptr, radius, allocVec3(normal).ptr, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawSphereNative(center: Vector3d, radius: Float, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_sphere(g, allocVec3(center).ptr, radius, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawGlobeNative(center: Vector3d, radius: Float, latLines: Int, lonLines: Int, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_globe(g, allocVec3(center).ptr, radius, latLines.toUInt(), lonLines.toUInt(), allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawAabbNative(min: Vector3d, max: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_aabb(g, allocVec3(min).ptr, allocVec3(max).ptr, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawObbNative(center: Vector3d, halfExtents: Vector3d, rotation: Quaterniond, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_obb(g, allocVec3(center).ptr, allocVec3(halfExtents).ptr, allocQuat(rotation).ptr, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawCapsuleNative(a: Vector3d, b: Vector3d, radius: Float, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_capsule(g, allocVec3(a).ptr, allocVec3(b).ptr, radius, allocColour(colour).ptr)
}

internal actual fun DebugDraw.drawConeNative(apex: Vector3d, dir: Vector3d, angle: Float, length: Float, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    dropbear_debug_draw_cone(g, allocVec3(apex).ptr, allocVec3(dir).ptr, angle, length, allocColour(colour).ptr)
}


internal actual fun DebugDraw.drawLineNative(start: Vector3d, end: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val s = allocVec3(start); val e = allocVec3(end)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_line(g, s.ptr, e.ptr, c)
}

internal actual fun DebugDraw.drawRayNative(origin: Vector3d, dir: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val o = allocVec3(origin); val d = allocVec3(dir)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_ray(g, o.ptr, d.ptr, c)
}

internal actual fun DebugDraw.drawArrowNative(start: Vector3d, end: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val s = allocVec3(start); val e = allocVec3(end)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_arrow(g, s.ptr, e.ptr, c)
}

internal actual fun DebugDraw.drawPointNative(pos: Vector3d, size: Float, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val p = allocVec3(pos)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_point(g, p.ptr, size, c)
}

internal actual fun DebugDraw.drawCircleNative(center: Vector3d, radius: Float, normal: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val ct = allocVec3(center); val n = allocVec3(normal)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_circle(g, ct.ptr, radius, n.ptr, c)
}

internal actual fun DebugDraw.drawSphereNative(center: Vector3d, radius: Float, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val ct = allocVec3(center)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_sphere(g, ct.ptr, radius, c)
}

internal actual fun DebugDraw.drawGlobeNative(center: Vector3d, radius: Float, latLines: Int, lonLines: Int, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val ct = allocVec3(center)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_globe(g, ct.ptr, radius, latLines.toUInt(), lonLines.toUInt(), c)
}

internal actual fun DebugDraw.drawAabbNative(min: Vector3d, max: Vector3d, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val mn = allocVec3(min); val mx = allocVec3(max)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_aabb(g, mn.ptr, mx.ptr, c)
}

internal actual fun DebugDraw.drawObbNative(center: Vector3d, halfExtents: Vector3d, rotation: Quaterniond, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val ct = allocVec3(center); val he = allocVec3(halfExtents)
    val rot = allocQuat(rotation)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_obb(g, ct.ptr, he.ptr, rot.ptr, c)
}

internal actual fun DebugDraw.drawCapsuleNative(a: Vector3d, b: Vector3d, radius: Float, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val va = allocVec3(a); val vb = allocVec3(b)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_capsule(g, va.ptr, vb.ptr, radius, c)
}

internal actual fun DebugDraw.drawConeNative(apex: Vector3d, dir: Vector3d, angle: Float, length: Float, colour: Colour) = memScoped {
    val g = DropbearEngine.native.graphicsContextHandle ?: return@memScoped
    val a = allocVec3(apex); val d = allocVec3(dir)
    val c = nColour(colour).toCValue(this)
    dropbear_debug_draw_cone(g, a.ptr, d.ptr, angle, length, c)
}
