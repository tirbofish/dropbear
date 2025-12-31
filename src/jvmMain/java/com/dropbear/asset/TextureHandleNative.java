package com.dropbear.asset;

import com.dropbear.EucalyptusCoreLoader;

public class TextureHandleNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native String getTextureName(long assetRegistryHandle, long handle);
}
