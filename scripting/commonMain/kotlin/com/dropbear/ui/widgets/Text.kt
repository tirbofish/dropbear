package com.dropbear.ui.widgets

import com.dropbear.ui.Align
import com.dropbear.ui.UIElement
import com.dropbear.utils.Colour

class Text : UIElement() {
    var content: String = "empty..."
    var align: Align? = null
    var fontSize: Double = 12.0
    var color: Colour = Colour.BLACK
}