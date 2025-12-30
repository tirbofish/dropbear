package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;

public class RigidBodyNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native Index rigidBodyExistsForEntity(long physicsPtr, long entityId);

    public static native int getRigidBodyMode(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyMode(long physicsPtr, RigidBody rigidBody, int mode);

    public static native double getRigidBodyGravityScale(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyGravityScale(long physicsPtr, RigidBody rigidBody, double gravityScale);

    public static native double getRigidBodyLinearDamping(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyLinearDamping(long physicsPtr, RigidBody rigidBody, double linearDamping);

    public static native double getRigidBodyAngularDamping(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyAngularDamping(long physicsPtr, RigidBody rigidBody, double angularDamping);

    public static native boolean getRigidBodyCanSleep(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyCanSleep(long physicsPtr, RigidBody rigidBody, boolean canSleep);

    public static native boolean getRigidBodyCcdEnabled(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyCcdEnabled(long physicsPtr, RigidBody rigidBody, boolean ccdEnabled);

    public static native Vector3d getRigidBodyLinearVelocity(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyLinearVelocity(long physicsPtr, RigidBody rigidBody, Vector3d linearVelocity);

    public static native Vector3d getRigidBodyAngularVelocity(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyAngularVelocity(long physicsPtr, RigidBody rigidBody, Vector3d angularVelocity);

    public static native AxisLock getRigidBodyLockTranslation(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyLockTranslation(long physicsPtr, RigidBody rigidBody, AxisLock lockTranslation);

    public static native AxisLock getRigidBodyLockRotation(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyLockRotation(long physicsPtr, RigidBody rigidBody, AxisLock lockRotation);

    public static native long[] getRigidBodyChildren(long physicsPtr, RigidBody rigidBody);
    public static native void setRigidBodyChildren(long physicsPtr, RigidBody rigidBody, long[] children);

    public static native void applyImpulse(long physicsPtr, RigidBody rigidBody, double x, double y, double z);
    public static native void applyTorqueImpulse(long physicsPtr, RigidBody rigidBody, double x, double y, double z);
}