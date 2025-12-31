package com.dropbear;

public class EntityRefNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native String getEntityLabel(long worldPtr, long entityId);
    public static native long[] getChildren(long worldPtr, long entityId);
    public static native Long getChildByLabel(long worldPtr, long entityId, String label);
    public static native Long getParent(long worldPtr, long entityId);
}
