package com.dropbear.ffi.components;

import com.dropbear.EntityTransform;
import com.dropbear.NativeEngineLoader;
import com.dropbear.math.Transform;

public class EntityTransformNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native EntityTransform getTransform(long handle, long entityHandle);
    public static native Transform propagateTransform(long worldHandle, long id);
    public static native void setTransform(long worldHandle, long id, EntityTransform transform);
}
