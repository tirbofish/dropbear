package com.dropbear.components;

import com.dropbear.EucalyptusCoreLoader;

public class LabelNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean labelExistsForEntity(long worldHandle, long entityId);
}