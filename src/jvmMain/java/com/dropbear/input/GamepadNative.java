package com.dropbear.input;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector2d;

public class GamepadNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean isGamepadButtonPressed(long inputStateHandle, long gamepad, int gamepadButton);
    public static native Vector2d getLeftStickPosition(long inputStateHandle, long gamepad);
    public static native Vector2d getRightStickPosition(long inputStateHandle, long gamepad);
}
