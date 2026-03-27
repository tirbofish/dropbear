package com.dropbear.rendering;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Quaterniond;
import com.dropbear.math.Vector3d;
import com.dropbear.utils.Colour;

public class DebugDrawNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native void drawLine(long graphicsContextPtr, Vector3d start, Vector3d end, Colour colour);
    public static native void drawRay(long graphicsContextPtr, Vector3d origin, Vector3d dir, Colour colour);
    public static native void drawArrow(long graphicsContextPtr, Vector3d start, Vector3d end, Colour colour);
    public static native void drawPoint(long graphicsContextPtr, Vector3d pos, float size, Colour colour);
    public static native void drawCircle(long graphicsContextPtr, Vector3d center, float radius, Vector3d normal, Colour colour);
    public static native void drawSphere(long graphicsContextPtr, Vector3d center, float radius, Colour colour);
    public static native void drawGlobe(long graphicsContextPtr, Vector3d center, float radius, int latLines, int lonLines, Colour colour);
    public static native void drawAabb(long graphicsContextPtr, Vector3d min, Vector3d max, Colour colour);
    public static native void drawObb(long graphicsContextPtr, Vector3d center, Vector3d halfExtents, Quaterniond rotation, Colour colour);
    public static native void drawCapsule(long graphicsContextPtr, Vector3d a, Vector3d b, float radius, Colour colour);
    public static native void drawCylinder(long graphicsContextPtr, Vector3d center, float halfHeight, float radius, Vector3d axis, Colour colour);
    public static native void drawCone(long graphicsContextPtr, Vector3d apex, Vector3d dir, float angle, float length, Colour colour);
}
