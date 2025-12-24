package com.dropbear.ffi;

import com.dropbear.NativeEngineLoader;
import com.dropbear.scene.SceneLoadHandle;
import com.dropbear.scene.SceneLoadStatus;
import com.dropbear.utils.Progress;

public class SceneNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native SceneLoadHandle loadSceneAsync(long commandBufferHandle, long sceneLoader, String sceneName);
    public static native SceneLoadHandle loadSceneAsync(long commandBufferHandle, long sceneLoader, String sceneName, String loadingScene);
    public static native void switchToSceneAsync(long commandBufferHandle, SceneLoadHandle handle);
    public static native void switchToSceneImmediate(long commandBufferHandle, String sceneName);
    public static native Progress getSceneLoadProgress(long sceneLoader, SceneLoadHandle handle);
    public static native SceneLoadStatus getSceneLoadStatus(long sceneLoader, SceneLoadHandle handle);
}