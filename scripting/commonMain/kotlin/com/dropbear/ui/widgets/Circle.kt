package com.dropbear.ui.widgets

import com.dropbear.ui.Align
import com.dropbear.ui.UIContainer
import com.dropbear.ui.UIElement

class Circle : UIElement(), UIContainer {
    var radius: Double = 0.0
    var alignment: Align? = null
    private val children = mutableListOf<UIElement>()

    fun align(block: Align.() -> Unit) {
        alignment = Align().apply(block)
    }

    override fun addChild(child: UIElement) {
        child.parent = this
        children.add(child)
    }

    override fun getChildren(): List<UIElement> = children.toList()
}