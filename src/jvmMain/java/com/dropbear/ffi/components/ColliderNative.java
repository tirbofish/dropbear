package com.dropbear.ffi.components;

import com.dropbear.physics.Collider;

public class ColliderNative {
    static {
        com.dropbear.NativeEngineLoader.ensureLoaded();
    }

    public static native void setCollider(long physicsEngineHandle, Collider collider);
}
