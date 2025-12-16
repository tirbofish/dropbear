package com.dropbear.ffi;

import com.dropbear.NativeEngineLoader;
import com.dropbear.scene.SceneLoadHandle;
import com.dropbear.utils.Progress;

public class SceneNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native SceneLoadHandle loadSceneAsync(long commandBufferHandle, String sceneName);
    public static native SceneLoadHandle loadSceneAsync(long commandBufferHandle, String sceneName, String loadingScene);
    public static native int switchToSceneAsync(long commandBufferHandle, SceneLoadHandle handle);
    public static native void switchToSceneImmediate(long commandBufferHandle, String sceneName);
    public static native Progress getSceneLoadProgress(long commandBufferHandle, SceneLoadHandle handle);
}
