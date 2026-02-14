package com.dropbear.ui

class Response(val widgetId: WidgetId) {
    val clicked: Boolean
        get() = getClicked()

    val hovering: Boolean
        get() = getHovering()
}

expect fun Response.getClicked(): Boolean
expect fun Response.getHovering(): Boolean