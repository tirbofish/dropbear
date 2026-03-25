package com.dropbear.ui

import com.dropbear.ui.Response

// UI response queries have no C API yet; always return non-interacted state
actual fun Response.getClicked(): Boolean = false
actual fun Response.getHovering(): Boolean = false