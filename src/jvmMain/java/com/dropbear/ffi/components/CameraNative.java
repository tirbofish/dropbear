package com.dropbear.ffi.components;

import com.dropbear.Camera;
import com.dropbear.NativeEngineLoader;

public class CameraNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native Camera getCamera(long worldHandle, String label);
    public static native Camera getAttachedCamera(long worldHandle, long entityHandle);
    public static native void setCamera(long worldHandle, Camera camera);
}
