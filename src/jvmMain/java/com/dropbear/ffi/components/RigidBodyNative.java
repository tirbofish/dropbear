package com.dropbear.ffi.components;

import com.dropbear.physics.Collider;
import com.dropbear.physics.Index;
import com.dropbear.physics.RigidBody;

public class RigidBodyNative {
    static {
        com.dropbear.NativeEngineLoader.ensureLoaded();
    }

    public static native void applyImpulse(long physicsEngineHandle, Index rigidBodyId, double x, double y, double z);
    public static native void applyTorqueImpulse(long physicsEngineHandle, Index rigidBodyId, double x, double y, double z);

    public static native void setRigidBody(long worldHandle, long physicsEngineHandle, RigidBody rigidBody);
    public static native Collider[] getChildColliders(long worldHandle, long physicsEngineHandle, Index rigidBodyId);
}