package io.github.tirbofish.surfaceNets;

import com.dropbear.ffi.DynamicLibraryLoader;
import com.dropbear.logging.Logger;

public class SurfaceNetsDylibLoader implements DynamicLibraryLoader {
    private static boolean loaded = false;

    public void ensureLoaded() {
        if (!loaded) {
            Logger.info("Initialising \"libsurface_nets_plugin\"", "SurfaceNetsDylibLoader::ensureLoaded");
            System.loadLibrary("surface_nets_plugin");
            Logger.info("Loaded!", "SurfaceNetsDylibLoader::ensureLoaded");
            loaded = true;
        }

    }

    public boolean isLoaded() {
        return loaded;
    }
}
