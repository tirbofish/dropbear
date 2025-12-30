package com.dropbear.input;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector2d;

public class InputStateNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native void printInputState(long inputStateHandle);
    public static native boolean isKeyPressed(long inputStateHandle, int keyCode);
    public static native Vector2d getMousePosition(long inputStateHandle);
    public static native boolean isMouseButtonPressed(long inputStateHandle, MouseButton mouseButton);
    public static native Vector2d getMouseDelta(long inputStateHandle);
    public static native boolean isCursorLocked(long inputStateHandle);
    public static native void setCursorLocked(long commandBufferPtr, long inputStateHandle, boolean locked);
    public static native Vector2d getLastMousePos(long inputStateHandle);
    public static native boolean isCursorHidden(long inputStateHandle);
    public static native void setCursorHidden(long commandBufferPtr, long inputStateHandle, boolean hidden);
    public static native long[] getConnectedGamepads(long inputStateHandle);
}
