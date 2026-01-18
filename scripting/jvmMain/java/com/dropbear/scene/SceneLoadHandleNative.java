package com.dropbear.scene;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.utils.Progress;

public class SceneLoadHandleNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native String getSceneLoadHandleSceneName(long sceneLoaderHandle, long sceneId);
    public static native void switchToSceneAsync(long commandBufferPtr, long sceneLoaderHandle, long sceneId);
    public static native Progress getSceneLoadProgress(long sceneLoaderHandle, long sceneId);
    public static native int getSceneLoadStatus(long sceneLoaderHandle, long sceneId);
}
