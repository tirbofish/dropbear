package com.dropbear.asset;

import com.dropbear.EucalyptusCoreLoader;

public class AssetHandleNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean isModelHandle(long assetRegistryHandle, long handle);
    public static native boolean isTextureHandle(long assetRegistryHandle, long handle);
}
