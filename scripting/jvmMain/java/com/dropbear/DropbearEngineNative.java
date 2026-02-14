package com.dropbear;

public class DropbearEngineNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native long getEntity(long worldPtr, String label);
    public static native long getAsset(long assetHandle, String label);
    public static native void quit(long commandBufferPtr);
}
