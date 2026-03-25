package com.dropbear.ui.widgets

import com.dropbear.ui.Response

actual fun Rectangle.getResponse(): Response = Response(id)