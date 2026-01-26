package com.dropbear.ui.styling

import com.dropbear.ui.styling.fonts.FontAttributes
import com.dropbear.ui.styling.fonts.FontName
import com.dropbear.ui.styling.fonts.TextAlignment
import com.dropbear.utils.Colour

data class TextStyle(
    var font: FontName,
    var fontSize: Double,
    var colour: Colour,
    var align: TextAlignment,
    var attrs: FontAttributes,
) {
    companion object {
        fun label(): TextStyle {
            return TextStyle(
                font = FontName("default"),
                fontSize = 14.0,
                colour = Colour.WHITE,
                align = TextAlignment.Start,
                attrs = FontAttributes()
            )
        }
    }

    override fun toString(): String {
        return "TextStyle(font=$font, fontSize=$fontSize, colour=$colour)"
    }
}