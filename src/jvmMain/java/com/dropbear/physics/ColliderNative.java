package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;

public class ColliderNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native ColliderShape getColliderShape(long physicsPtr, Collider collider);
    public static native void setColliderShape(long physicsPtr, Collider collider, ColliderShape shape);

    public static native double getColliderDensity(long physicsPtr, Collider collider);
    public static native void setColliderDensity(long physicsPtr, Collider collider, double density);

    public static native double getColliderFriction(long physicsPtr, Collider collider);
    public static native void setColliderFriction(long physicsPtr, Collider collider, double friction);

    public static native double getColliderRestitution(long physicsPtr, Collider collider);
    public static native void setColliderRestitution(long physicsPtr, Collider collider, double restitution);

    public static native double getColliderMass(long physicsPtr, Collider collider);
    public static native void setColliderMass(long physicsPtr, Collider collider, double mass);

    public static native boolean getColliderIsSensor(long physicsPtr, Collider collider);
    public static native void setColliderIsSensor(long physicsPtr, Collider collider, boolean isSensor);

    public static native Vector3d getColliderTranslation(long physicsPtr, Collider collider);
    public static native void setColliderTranslation(long physicsPtr, Collider collider, Vector3d translation);

    public static native Vector3d getColliderRotation(long physicsPtr, Collider collider);
    public static native void setColliderRotation(long physicsPtr, Collider collider, Vector3d rotation);
}