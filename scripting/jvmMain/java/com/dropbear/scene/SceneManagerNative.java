package com.dropbear.scene;

import com.dropbear.EucalyptusCoreLoader;

public class SceneManagerNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native long loadSceneAsync(long commandBufferPtr, long sceneManagerHandle, String sceneName);
    public static native long loadSceneAsyncWithLoading(long commandBufferPtr, long sceneManagerHandle, String sceneName, String loadingScene);
    public static native void switchToSceneImmediate(long commandBufferPtr, String sceneName);
}