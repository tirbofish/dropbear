package com.dropbear.components;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;

public class CameraNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean cameraExistsForEntity(long worldHandle, long entityId);

    public static native Vector3d getCameraEye(long worldHandle, long entityId);
    public static native void setCameraEye(long worldHandle, long entityId, Vector3d value);

    public static native Vector3d getCameraTarget(long worldHandle, long entityId);
    public static native void setCameraTarget(long worldHandle, long entityId, Vector3d value);

    public static native Vector3d getCameraUp(long worldHandle, long entityId);
    public static native void setCameraUp(long worldHandle, long entityId, Vector3d value);

    public static native double getCameraAspect(long worldHandle, long entityId);

    public static native double getCameraFovY(long worldHandle, long entityId);
    public static native void setCameraFovY(long worldHandle, long entityId, double value);

    public static native double getCameraZNear(long worldHandle, long entityId);
    public static native void setCameraZNear(long worldHandle, long entityId, double value);

    public static native double getCameraZFar(long worldHandle, long entityId);
    public static native void setCameraZFar(long worldHandle, long entityId, double value);

    public static native double getCameraYaw(long worldHandle, long entityId);
    public static native void setCameraYaw(long worldHandle, long entityId, double value);

    public static native double getCameraPitch(long worldHandle, long entityId);
    public static native void setCameraPitch(long worldHandle, long entityId, double value);

    public static native double getCameraSpeed(long worldHandle, long entityId);
    public static native void setCameraSpeed(long worldHandle, long entityId, double value);

    public static native double getCameraSensitivity(long worldHandle, long entityId);
    public static native void setCameraSensitivity(long worldHandle, long entityId, double value);
}