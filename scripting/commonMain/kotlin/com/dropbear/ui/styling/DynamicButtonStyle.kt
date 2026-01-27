package com.dropbear.ui.styling

import com.dropbear.ui.styling.fonts.TextAlignment
import com.dropbear.utils.Colour

data class DynamicButtonStyle(
    var text: TextStyle = TextStyle(align = TextAlignment.Center),
    var fill: Colour = Colour.GRAY,
    var border: Border? = null,
)