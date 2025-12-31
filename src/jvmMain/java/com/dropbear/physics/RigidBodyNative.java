package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;

public class RigidBodyNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native Index rigidBodyExistsForEntity(long worldPtr, long physicsPtr, long entityId);

    public static native int getRigidBodyMode(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyMode(long worldPtr, long physicsPtr, RigidBody rigidBody, int mode);

    public static native double getRigidBodyGravityScale(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyGravityScale(long worldPtr, long physicsPtr, RigidBody rigidBody, double gravityScale);

    public static native double getRigidBodyLinearDamping(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyLinearDamping(long worldPtr, long physicsPtr, RigidBody rigidBody, double linearDamping);

    public static native double getRigidBodyAngularDamping(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyAngularDamping(long worldPtr, long physicsPtr, RigidBody rigidBody, double angularDamping);

    public static native boolean getRigidBodySleep(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodySleep(long worldPtr, long physicsPtr, RigidBody rigidBody, boolean canSleep);

    public static native boolean getRigidBodyCcdEnabled(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyCcdEnabled(long worldPtr, long physicsPtr, RigidBody rigidBody, boolean ccdEnabled);

    public static native Vector3d getRigidBodyLinearVelocity(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyLinearVelocity(long worldPtr, long physicsPtr, RigidBody rigidBody, Vector3d linearVelocity);

    public static native Vector3d getRigidBodyAngularVelocity(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyAngularVelocity(long worldPtr, long physicsPtr, RigidBody rigidBody, Vector3d angularVelocity);

    public static native AxisLock getRigidBodyLockTranslation(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyLockTranslation(long worldPtr, long physicsPtr, RigidBody rigidBody, AxisLock lockTranslation);

    public static native AxisLock getRigidBodyLockRotation(long worldPtr, long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyLockRotation(long worldPtr, long physicsPtr, RigidBody rigidBody, AxisLock lockRotation);

    public static native Collider[] getRigidBodyChildren(long worldPtr, long physicsPtr, RigidBody rigidBody);

    public static native void applyImpulse(long worldPtr, long physicsPtr, RigidBody rigidBody, double x, double y, double z);
    public static native void applyTorqueImpulse(long worldPtr, long physicsPtr, RigidBody rigidBody, double x, double y, double z);
}