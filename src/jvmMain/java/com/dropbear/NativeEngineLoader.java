package com.dropbear;

import java.lang.System;

public class NativeEngineLoader {
    private static boolean loaded = false;

    /// Ensures that the eucalyptus_core library is loaded before accessing
    /// any of the JNI static functions.
    public static void ensureLoaded() {
        if (!loaded) {
            System.loadLibrary("eucalyptus_core");
            loaded = true;
        }
    }
}
