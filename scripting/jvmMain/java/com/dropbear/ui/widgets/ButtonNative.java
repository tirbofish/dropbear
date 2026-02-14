package com.dropbear.ui.widgets;

import com.dropbear.EucalyptusCoreLoader;

public class ButtonNative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native boolean getClicked(long uiBufferHandle, long id);
    public static native boolean getHovering(long uiBufferHandle, long id);
}
