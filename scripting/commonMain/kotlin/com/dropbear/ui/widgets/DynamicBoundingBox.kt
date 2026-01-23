package com.dropbear.ui.widgets

import com.dropbear.ui.UIContainer
import com.dropbear.ui.UIElement

class DynamicBoundingBox: UIElement(), UIContainer {
    private val children = mutableListOf<UIElement>()

    override fun addChild(child: UIElement) {
        child.parent = this
        children.add(child)
    }

    override fun getChildren(): List<UIElement> = children

    fun findByName(name: String): UIElement? {
        if (this.name == name) return this
        for (child in children) {
            if (child.name == name) return child
            if (child is UIContainer) {
                if (child is DynamicBoundingBox) {
                    val found = child.findByName(name)
                    if (found != null) return found
                }
            }
        }
        return null
    }
}