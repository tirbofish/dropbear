package com.dropbear.physics;

import com.dropbear.EucalyptusCoreLoader;
import java.util.List;

public class ColliderGroupNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean colliderGroupExistsForEntity(long worldPtr, long entityId);
    public static native List<Collider> getColliderGroupColliders(long worldPtr, long physicsPtr, long entityId);
}