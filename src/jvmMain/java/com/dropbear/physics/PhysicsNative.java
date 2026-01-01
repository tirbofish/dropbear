package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;

public class PhysicsNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native Vector3d getGravity(long physicsHandle);
    public static native void setGravity(long physicsHandle, Vector3d gravity);

    public static native RayHit raycast(long physicsHandle, Vector3d origin, Vector3d direction, double toi, boolean solid);
    public static native boolean isOverlapping(long physicsHandle, Collider collider1, Collider collider2);
    public static native boolean isTriggering(long physicsHandle, Collider collider1, Collider collider2);
    public static native boolean isTouching(long physicsHandle, long entity1, long entity2);
}
