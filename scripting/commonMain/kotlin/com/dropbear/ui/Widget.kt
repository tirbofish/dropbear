package com.dropbear.ui

/**
 * Specifies a type of widget that is available. This class aims to create a more "unified" experience for
 * describing a widget such as a button, or even a column.
 */
abstract class Widget {
    abstract val id: WidgetId
}