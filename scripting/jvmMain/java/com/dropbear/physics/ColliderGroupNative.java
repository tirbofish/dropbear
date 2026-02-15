package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;

public class ColliderGroupNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean colliderGroupExistsForEntity(long worldPtr, long entityId);
    public static native Collider[] getColliderGroupColliders(long worldPtr, long physicsPtr, long entityId);
}