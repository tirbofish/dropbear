package com.dropbear.ffi.components;

import com.dropbear.NativeEngineLoader;

public class HierarchyNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native long[] getChildren(long worldHandle, long entityId);
    public static native long getChildByLabel(long worldHandle, long entityId, String label);
    public static native long getParent(long worldHandle, long entityId);
}
