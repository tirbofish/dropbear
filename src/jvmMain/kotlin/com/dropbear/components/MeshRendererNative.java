package com.dropbear.components;

import com.dropbear.EucalyptusCoreLoader;

public class MeshRendererNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean meshRendererExistsForEntity(long worldHandle, long entityId);

    public static native long getModel(long worldHandle, long entityId);
    public static native void setModel(long worldHandle, long assetRegistryHandle, long entityId, long modelHandle);
    public static native long[] getAllTextureIds(long worldHandle, long assetRegistryHandle, long entityId);
    public static native Long getTexture(long worldHandle, long assetRegistryHandle, long entityId, String materialName);
    public static native void setTextureOverride(long worldHandle, long assetRegistryHandle, long entityId, String meshName, long textureHandle);
}