package com.dropbear.ffi;

import com.dropbear.NativeEngineLoader;
import com.dropbear.scene.SceneLoadHandle;
import com.dropbear.utils.Progress;

public class SceneNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    public static native SceneLoadHandle loadSceneAsync(String sceneName);
    public static native SceneLoadHandle loadSceneAsync(String sceneName, String loadingScene);
    public static native int switchToSceneAsync(SceneLoadHandle handle);
    public static native void switchToSceneImmediate(String sceneName);
    public static native Progress getSceneLoadProgress(SceneLoadHandle handle);
}
