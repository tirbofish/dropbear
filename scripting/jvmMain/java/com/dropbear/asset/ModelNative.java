package com.dropbear.asset;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.asset.model.*;

public class ModelNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native String getLabel(long assetManagerHandle, String modelName);
    public static native Mesh[] getMeshes(long assetManagerHandle, long modelHandle);
    public static native Material[] getMaterials(long assetManagerHandle, long modelHandle);
    public static native Skin[] getSkins(long assetManagerHandle, long modelHandle);
    public static native Animation[] getAnimations(long assetManagerHandle, long modelHandle);
    public static native Node[] getNodes(long assetManagerHandle, long modelHandle);
}
