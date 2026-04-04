package io.github.tirbofish.surfaceNets;

public class SurfaceNetsNative {
    static {
        new SurfaceNetsDylibLoader().ensureLoaded();
    }

    public static native boolean surfaceNetsExistsForEntity(long worldHandle, long entityId);
}
