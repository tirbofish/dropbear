package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;

public class ColliderGroupNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native Collider[] getColliderGroupColliders(long physicsPtr, ColliderGroup colliderGroup);
    public static native boolean colliderGroupExistsForEntity(long worldPtr, long entityId);
}