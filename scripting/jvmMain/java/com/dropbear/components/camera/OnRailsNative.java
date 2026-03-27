package com.dropbear.components.camera;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.math.Vector3d;

public class OnRailsNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean existsForEntity(long worldPtr, long entityId);

    public static native boolean getEnabled(long worldPtr, long entityId);
    public static native void setEnabled(long worldPtr, long entityId, boolean enabled);

    public static native float getProgress(long worldPtr, long entityId);
    public static native void setProgress(long worldPtr, long entityId, float progress);

    public static native int getPathLen(long worldPtr, long entityId);
    public static native Vector3d getPathPoint(long worldPtr, long entityId, int index);
    public static native void clearPath(long worldPtr, long entityId);
    public static native void pushPathPoint(long worldPtr, long entityId, Vector3d point);

    // 0=Automatic, 1=FollowEntity, 2=AxisDriven, 3=Manual
    public static native int getDriveType(long worldPtr, long entityId);

    public static native float getDriveAutomaticSpeed(long worldPtr, long entityId);
    public static native boolean getDriveAutomaticLooping(long worldPtr, long entityId);

    public static native long getDriveFollowEntityTarget(long worldPtr, long entityId);
    public static native boolean getDriveFollowEntityMonotonic(long worldPtr, long entityId);

    public static native long getDriveAxisDrivenTarget(long worldPtr, long entityId);
    public static native Vector3d getDriveAxisDrivenAxis(long worldPtr, long entityId);
    public static native float getDriveAxisDrivenRangeMin(long worldPtr, long entityId);
    public static native float getDriveAxisDrivenRangeMax(long worldPtr, long entityId);

    public static native void setDriveAutomatic(long worldPtr, long entityId, float speed, boolean looping);
    public static native void setDriveFollowEntity(long worldPtr, long entityId, long target, boolean monotonic);
    public static native void setDriveAxisDriven(long worldPtr, long entityId, long target, Vector3d axis, float rangeMin, float rangeMax);
    public static native void setDriveManual(long worldPtr, long entityId);
}
