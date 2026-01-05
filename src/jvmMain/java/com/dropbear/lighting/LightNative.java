package com.dropbear.lighting;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;
import com.dropbear.utils.Colour;
import com.dropbear.utils.Range;

public class LightNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean lightExistsForEntity(long worldHandle, long entityId);

    public static native Vector3d getPosition(long worldHandle, long entityId);
    public static native void setPosition(long worldHandle, long entityId, Vector3d position);
    public static native Vector3d getDirection(long worldHandle, long entityId);
    public static native void setDirection(long worldHandle, long entityId, Vector3d direction);
    public static native Colour getColour(long worldHandle, long entityId);
    public static native void setColour(long worldHandle, long entityId, Colour colour);
    public static native int getLightType(long worldHandle, long entityId);
    public static native void setLightType(long worldHandle, long entityId, int lightType);
    public static native double getIntensity(long worldHandle, long entityId);
    public static native void setIntensity(long worldHandle, long entityId, double intensity);
    public static native Attenuation getAttenuation(long worldHandle, long entityId);
    public static native void setAttenuation(long worldHandle, long entityId, Attenuation attenuation);
    public static native boolean getEnabled(long worldHandle, long entityId);
    public static native void setEnabled(long worldHandle, long entityId, boolean enabled);
    public static native double getCutoffAngle(long worldHandle, long entityId);
    public static native void setCutoffAngle(long worldHandle, long entityId, double cutoffAngle);
    public static native double getOuterCutoffAngle(long worldHandle, long entityId);
    public static native void setOuterCutoffAngle(long worldHandle, long entityId, double outerCutoffAngle);
    public static native boolean getCastsShadows(long worldHandle, long entityId);
    public static native void setCastsShadows(long worldHandle, long entityId, boolean castsShadows);
    public static native Range getDepth(long worldHandle, long entityId);
    public static native void setDepth(long worldHandle, long entityId, Range depth);
}
