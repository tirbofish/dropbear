package com.dropbear.ffi;

import com.dropbear.NativeEngineLoader;

public class InputStateNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    // input
    public static native void printInputState(long inputHandle);
    public static native boolean isKeyPressed(long inputHandle, int ordinal);
    public static native float[] getMousePosition(long inputHandle);
    public static native boolean isMouseButtonPressed(long inputHandle, int ordinal);
    public static native float[] getMouseDelta(long inputHandle);
    public static native boolean isCursorLocked(long inputHandle);
    public static native void setCursorLocked(long inputHandle, long graphicsHandle, boolean locked);
    public static native float[] getLastMousePos(long inputHandle);
    public static native boolean isCursorHidden(long inputHandle);
    public static native void setCursorHidden(long inputHandle, long graphicsHandle, boolean hidden);
    public static native String[] getAllTextures(long worldHandle, long entityHandle);
}
