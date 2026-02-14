package com.dropbear.ui;

import com.dropbear.EucalyptusCoreLoader;

public class UINative {
    static {
        new EucalyptusCoreLoader().ensureLoaded();
    }

    public static native void renderUI(long uiBufHandle, UIInstruction[] instructions);
}