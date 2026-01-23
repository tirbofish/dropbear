package com.dropbear.ui.widgets

import com.dropbear.math.Vector2d
import com.dropbear.ui.Align
import com.dropbear.ui.UIContainer
import com.dropbear.ui.UIElement
import com.dropbear.ui.styling.StyleConfig

class Rectangle : UIElement(), UIContainer {
    var size: Vector2d = Vector2d(0.0, 0.0)
    var align: Align? = null
    var styleConfig: StyleConfig? = null

    private val children = mutableListOf<UIElement>()

    fun style(block: StyleConfig.() -> Unit) {
        styleConfig = StyleConfig().apply(block)
    }

    override fun addChild(child: UIElement) {
        child.parent = this
        children.add(child)
    }

    override fun getChildren(): List<UIElement> = children.toList()
}