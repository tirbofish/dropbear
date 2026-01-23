package com.dropbear.ui

import com.dropbear.ui.widgets.*

interface UIContainer {
    fun addChild(child: UIElement)
    fun getChildren(): List<UIElement>

    fun Circle(block: Circle.() -> Unit) {
        val circle = Circle().apply(block)
        addChild(circle)
    }

    fun Rectangle(block: Rectangle.() -> Unit) {
        val rectangle = Rectangle().apply(block)
        addChild(rectangle)
    }

    fun Image(block: Image.() -> Unit) {
        val image = Image().apply(block)
        addChild(image)
    }

    fun Text(block: Text.() -> Unit) {
        val text = Text().apply(block)
        addChild(text)
    }

    fun DynamicBoundingBox(block: DynamicBoundingBox.() -> Unit) {
        val box = DynamicBoundingBox().apply(block)
        addChild(box)
    }
}