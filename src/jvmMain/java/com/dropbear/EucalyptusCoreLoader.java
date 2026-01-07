package com.dropbear;

import com.dropbear.ffi.DynamicLibraryLoader;
import com.dropbear.logging.Logger;

import java.lang.System;

public class EucalyptusCoreLoader implements DynamicLibraryLoader {
    private static boolean loaded = false;

    @Override
    public void ensureLoaded() {
        if (!loaded) {
            Logger.info("Initialising \"eucalyptus_core\"", "EucalyptusCoreLoader::ensureLoaded");
            System.loadLibrary("eucalyptus_core");
            Logger.info("Loaded!", "EucalyptusCoreLoader::ensureLoaded");
            loaded = true;
        }
    }

    @Override
    public boolean isLoaded() {
        return loaded;
    }
}
