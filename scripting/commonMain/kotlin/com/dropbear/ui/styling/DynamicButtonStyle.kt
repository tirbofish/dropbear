package com.dropbear.ui.styling

import com.dropbear.utils.Colour

data class DynamicButtonStyle(
    var text: TextStyle = TextStyle.label(),
    var fill: Colour = Colour.GRAY,
    var border: Border? = null,
)