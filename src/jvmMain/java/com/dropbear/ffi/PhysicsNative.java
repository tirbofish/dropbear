package com.dropbear.ffi;

import com.dropbear.NativeEngineLoader;
import com.dropbear.physics.Collider;
import com.dropbear.physics.RigidBody;

public class PhysicsNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native void setPhysicsEnabled(long physicsEngineHandle, long entityId, boolean enabled);
    public static native boolean isPhysicsEnabled(long physicsEngineHandle, long entityId);
    public static native RigidBody getRigidBody(long physicsEngineHandle, long entityId);
    public static native Collider[] getAllColliders(long physicsEngineHandle, long entityId);
}
