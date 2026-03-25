package com.dropbear.asset;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.asset.model.*;
import java.util.List;

public class ModelNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native String getLabel(long assetManagerHandle, String modelName);
    public static native List<Mesh> getMeshes(long assetManagerHandle, long modelHandle);
    public static native List<Material> getMaterials(long assetManagerHandle, long modelHandle);
    public static native List<Skin> getSkins(long assetManagerHandle, long modelHandle);
    public static native List<Animation> getAnimations(long assetManagerHandle, long modelHandle);
    public static native List<Node> getNodes(long assetManagerHandle, long modelHandle);
}
