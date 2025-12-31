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
}
