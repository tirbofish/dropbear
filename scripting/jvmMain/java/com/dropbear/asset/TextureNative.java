package com.dropbear.asset;

import com.dropbear.EucalyptusCoreLoader;

public class TextureNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }
    
    public static native String getLabel(long assetManagerHandle, long textureHandle);
    public static native int getWidth(long assetManagerHandle, long textureHandle);
    public static native int getHeight(long assetManagerHandle, long textureHandle);
}
