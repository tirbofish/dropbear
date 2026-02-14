package com.dropbear.ui.widgets;

import com.dropbear.EucalyptusCoreLoader;

public class CheckboxNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean getChecked(long uiBufferHandle, long id);
    public static native boolean hasCheckedState(long uiBufferHandle, long id);
}
