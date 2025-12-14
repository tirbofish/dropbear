package com.dropbear.ffi.components;

import com.dropbear.NativeEngineLoader;

public class LabelNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native String getEntityLabel(long worldHandle, long entityHandle);
}
