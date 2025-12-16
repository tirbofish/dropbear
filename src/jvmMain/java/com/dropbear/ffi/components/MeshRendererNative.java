package com.dropbear.ffi.components;

import com.dropbear.NativeEngineLoader;

public class MeshRendererNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    // model
    public static native long getModel(long worldHandle, long entityHandle);
    public static native void setModel(long worldHandle, long assetHandle, long entityHandle, long modelHandle);
    public static native boolean isModelHandle(long assetRegistryHandle, long handle);
    public static native boolean isUsingModel(long worldHandle, long entityHandle, long modelHandle);

    // texture
    public static native long getTexture(long worldHandle, long assetHandle, long entityHandle, String name);
    public static native String getTextureName(long assetHandle, long textureHandle);
    public static native void setTexture(long worldHandle, long assetRegistryHandle, long entityHandle,
                                         String oldMaterialName, long textureHandle);
    public static native boolean isTextureHandle(long assetRegistryHandle, long handle);
    public static native boolean isUsingTexture(long worldHandle, long entityHandle, long textureHandle);

    public static native String[] getAllTextures(long worldHandle, long entityHandle);
}
