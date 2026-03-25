package com.dropbear.ui.widgets

import com.dropbear.ui.Response

actual fun Text.getResponse(): Response = Response(id)