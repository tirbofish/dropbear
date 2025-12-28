package com.dropbear.ffi;

import com.dropbear.Camera;
import com.dropbear.EntityTransform;
import com.dropbear.NativeEngineLoader;
import com.dropbear.math.Transform;
import com.dropbear.scene.SceneLoadHandle;
import com.dropbear.utils.Progress;

/**
 * Describes all the functions that are available in
 * the `eucalyptus_core` dynamic library. 
 */
public class JNINative {
    static {
        NativeEngineLoader.ensureLoaded();
    }
    public static native long getEntity(long worldHandle, String label);
    public static native long getAsset(long assetRegistryHandle, String eucaURI);
}
