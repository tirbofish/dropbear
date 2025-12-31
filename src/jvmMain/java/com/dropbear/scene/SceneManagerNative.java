package com.dropbear.scene;

import com.dropbear.EucalyptusCoreLoader;

public class SceneManagerNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native Long loadSceneAsyncNative(long commandBufferPtr, long sceneManagerHandle, String sceneName);
    public static native Long loadSceneAsyncNative(long commandBufferPtr, long sceneManagerHandle, String sceneName, String loadingScene);
    public static native void switchToSceneImmediateNative(long commandBufferPtr, String sceneName);
}