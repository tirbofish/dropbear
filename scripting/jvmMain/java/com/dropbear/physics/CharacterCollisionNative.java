package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Transform;
import com.dropbear.math.Vector3d;

public class CharacterCollisionNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native Collider getCollider(long worldHandle, long entity, Index collisionHandle);
    public static native Transform getCharacterPosition(long worldHandle, long entity, Index collisionHandle);
    public static native Vector3d getTranslationApplied(long worldHandle, long entity, Index collisionHandle);
    public static native Vector3d getTranslationRemaining(long worldHandle, long entity, Index collisionHandle);
    public static native double getTimeOfImpact(long worldHandle, long entity, Index collisionHandle);
    public static native Vector3d getWitness1(long worldHandle, long entity, Index collisionHandle);
    public static native Vector3d getWitness2(long worldHandle, long entity, Index collisionHandle);
    public static native Vector3d getNormal1(long worldHandle, long entity, Index collisionHandle);
    public static native Vector3d getNormal2(long worldHandle, long entity, Index collisionHandle);
    public static native ShapeCastStatus getStatus(long worldHandle, long entity, Index collisionHandle);
}