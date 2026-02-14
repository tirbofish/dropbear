package com.dropbear.ui.styling

import com.dropbear.ui.styling.fonts.Family
import com.dropbear.ui.styling.fonts.FontAttributes
import com.dropbear.ui.styling.fonts.FontName
import com.dropbear.ui.styling.fonts.TextAlignment
import com.dropbear.utils.Colour

data class TextStyle(
    var fontSize: Double = 14.0,
    var lineHeightOverride: Double? = null,
    var colour: Colour = Colour.WHITE,
    var align: TextAlignment = TextAlignment.Start,
    var attrs: FontAttributes = FontAttributes(
        family = Family.SansSerif,
    ),
) {
    companion object {
        fun label(): TextStyle {
            return TextStyle()
        }
    }
}