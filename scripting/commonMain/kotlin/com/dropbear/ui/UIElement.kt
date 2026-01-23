package com.dropbear.ui

import com.dropbear.DropbearEngine

open class UIElement {
    var name: String = "noNamedUIElement"
    internal var parent: UIElement? = null

    var onClick: ((DropbearEngine, UIElement) -> Unit)? = null
    var onHover: ((DropbearEngine, UIElement) -> Unit)? = null
    var onHoverExit: ((DropbearEngine, UIElement) -> Unit)? = null
}