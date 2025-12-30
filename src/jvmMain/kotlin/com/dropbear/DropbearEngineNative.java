package com.dropbear;

public class DropbearEngineNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native Long getEntity(long worldPtr, String label);
    public static native Long getAsset(long worldPtr, String label);
    public static native void quit(long commandBufferPtr);
}
