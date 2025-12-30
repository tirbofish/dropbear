package com.dropbear;

import com.dropbear.ffi.DynamicLibraryLoader;

import java.lang.System;

public class EucalyptusCoreLoader implements DynamicLibraryLoader {
    private static boolean loaded = false;

    @Override
    public void ensureLoaded() {
        if (!loaded) {
            System.loadLibrary("eucalyptus_core");
            loaded = true;
        }
    }

    @Override
    public boolean isLoaded() {
        return loaded;
    }
}
