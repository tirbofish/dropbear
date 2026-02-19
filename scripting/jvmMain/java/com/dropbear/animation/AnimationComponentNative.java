package com.dropbear.animation;

import com.dropbear.EucalyptusCoreLoader;

public class AnimationComponentNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean animationComponentExistsForEntity(long worldHandle, long entityId);

    public static native Integer getActiveAnimationIndex(long worldHandle, long entityId);
    public static native void setActiveAnimationIndex(long worldHandle, long entityId, Integer index);
    public static native double getTime(long worldHandle, long entityId);
    public static native void setTime(long worldHandle, long entityId, double value);
    public static native double getSpeed(long worldHandle, long entityId);
    public static native void setSpeed(long worldHandle, long entityId, double value);
    public static native boolean getLooping(long worldHandle, long entityId);
    public static native void setLooping(long worldHandle, long entityId, boolean value);
    public static native boolean getIsPlaying(long worldHandle, long entityId);
    public static native void setIsPlaying(long worldHandle, long entityId, boolean value);
    public static native Integer getIndexFromString(long worldHandle, long entityId, String name);
    public static native String[] getAvailableAnimations(long worldHandle, long entityId);
}
