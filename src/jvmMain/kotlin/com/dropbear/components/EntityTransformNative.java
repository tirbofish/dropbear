package com.dropbear.components;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Transform;

public class EntityTransformNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean entityTransformExistsForEntity(long worldPtr, long entityId);

    public static native Transform getLocalTransform(long worldPtr, long entityId);
    public static native void setLocalTransform(long worldPtr, long entityId, Transform transform);
    public static native Transform getWorldTransform(long worldPtr, long entityId);
    public static native void setWorldTransform(long worldPtr, long entityId, Transform transform);
    public static native Transform propagateTransform(long worldPtr, long entityId);
}