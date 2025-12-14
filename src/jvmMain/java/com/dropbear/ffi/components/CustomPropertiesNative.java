package com.dropbear.ffi.components;

import com.dropbear.NativeEngineLoader;

public class CustomPropertiesNative {
    static {
        NativeEngineLoader.ensureLoaded();
    }

    // properties
    public static native String getStringProperty(long worldHandle, long entityHandle, String label);
    public static native int getIntProperty(long worldHandle, long entityHandle, String label);
    public static native long getLongProperty(long worldHandle, long entityHandle, String label);
    public static native double getFloatProperty(long worldHandle, long entityHandle, String label);
    public static native boolean getBoolProperty(long worldHandle, long entityHandle, String label);
    public static native float[] getVec3Property(long worldHandle, long entityHandle, String label);

    public static native void setStringProperty(long worldHandle, long entityHandle, String label, String value);
    public static native void setIntProperty(long worldHandle, long entityHandle, String label, int value);
    public static native void setLongProperty(long worldHandle, long entityHandle, String label, long value);
    public static native void setFloatProperty(long worldHandle, long entityHandle, String label, double value);
    public static native void setBoolProperty(long worldHandle, long entityHandle, String label, boolean value);
    public static native void setVec3Property(long worldHandle, long entityHandle, String label, float[] value);

}
