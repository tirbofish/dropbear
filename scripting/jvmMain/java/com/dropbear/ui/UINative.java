package com.dropbear.ui;

import com.dropbear.EucalyptusCoreLoader;
import com.dropbear.ui.primitive.Circle;
import com.dropbear.ui.primitive.Rectangle;

public class UINative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native void pushRect(long uiBufferHandle, Rectangle rect);
    public static native void pushCircle(long uiBufferHandle, Circle circle);
    public static native boolean wasClicked(long uiBufferHandle, long id);
}
