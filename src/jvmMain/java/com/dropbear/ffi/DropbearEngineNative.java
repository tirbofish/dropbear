package com.dropbear.ffi;

import com.dropbear.NativeEngineLoader;

public class DropbearEngineNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native void quit(long graphicsHandle);
}
