package com.dropbear.ui

/**
 * Marks a property of a [NativeComponent][com.dropbear.ecs.NativeComponent] subclass for
 * display in the editor's inspector panel.
 *
 * @param widgetType The widget to render for this property in the inspector.
 */
@Target(AnnotationTarget.PROPERTY, AnnotationTarget.FIELD)
@Retention(AnnotationRetention.RUNTIME)
annotation class Inspect(val widgetType: WidgetType)
