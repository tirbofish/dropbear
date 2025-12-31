package com.dropbear.components;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;

public class CustomPropertiesNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean customPropertiesExistsForEntity(long worldHandle, long entityId);

    public static native String getStringProperty(long worldHandle, long entityId, String label);
    public static native Integer getIntProperty(long worldHandle, long entityId, String label);
    public static native Long getLongProperty(long worldHandle, long entityId, String label);
    public static native Double getDoubleProperty(long worldHandle, long entityId, String label);
    public static native Float getFloatProperty(long worldHandle, long entityId, String label);
    public static native Boolean getBoolProperty(long worldHandle, long entityId, String label);
    public static native Vector3d getVec3Property(long worldHandle, long entityId, String label);

    public static native void setStringProperty(long worldHandle, long entityId, String label, String value);
    public static native void setIntProperty(long worldHandle, long entityId, String label, int value);
    public static native void setLongProperty(long worldHandle, long entityId, String label, long value);
    public static native void setFloatProperty(long worldHandle, long entityId, String label, double value);
    public static native void setBoolProperty(long worldHandle, long entityId, String label, boolean value);
    public static native void setVec3Property(long worldHandle, long entityId, String label, Vector3d value);
}