package com.dropbear.ui

/**
 * Specifies a type of widget that is available. This class aims to create a more "unified" experience for
 * describing a widget such as a button, or even a column.
 */
abstract class Widget {
    /**
     * The [WidgetId] used to differentiate between two different widgets.
     *
     * It is typically derived from the name of the widget.
     */
    abstract val id: WidgetId

    /**
     * Converts the [Widget] to a [UIInstruction] list, allowing to be added
     * by a [UIBuilder].
     */
    abstract fun toInstruction(): List<UIInstruction>
}