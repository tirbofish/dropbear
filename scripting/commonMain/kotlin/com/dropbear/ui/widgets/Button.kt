package com.dropbear.ui.widgets

import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.styling.DynamicButtonStyle
import com.dropbear.ui.styling.Padding
import com.dropbear.utils.Colour

class Button(
    var text: String,
    var padding: Padding,
    var borderRadius: Double,
    var style: DynamicButtonStyle,
    var hoverStyle: DynamicButtonStyle,
    var downStyle: DynamicButtonStyle,
): Widget() {
    override lateinit var id: WidgetId

    val clicked: Boolean
        get() = getClicked()

    val hovering: Boolean
        get() = getHovering()

    companion object {
        fun styled(text: String) : Button {
            val result = Button(
                text = text,
                padding = Padding.balanced(20.0, 10.0),
                borderRadius = 6.0,
                style = DynamicButtonStyle(fill = Colour.BACKGROUND_3),
                hoverStyle = DynamicButtonStyle(fill = Colour.BACKGROUND_3.adjust(1.2)),
                downStyle = DynamicButtonStyle(fill = Colour.BACKGROUND_3.adjust(0.8)),
            )

            result.id = WidgetId(result.hashCode().toLong())

            return result
        }

        fun unstyled(text: String) : Button {
            val result = Button(
                text = text,
                padding = Padding.zero(),
                borderRadius = 0.0,
                style = DynamicButtonStyle(),
                hoverStyle = DynamicButtonStyle(),
                downStyle = DynamicButtonStyle(),
            )

            result.id = WidgetId(result.hashCode().toLong())

            return result
        }
    }

    sealed class ButtonInstruction: UIInstruction {
        data class Button(val id: WidgetId, val button: com.dropbear.ui.widgets.Button) : ButtonInstruction()
    }

    fun toInstruction(): ButtonInstruction.Button {
        return ButtonInstruction.Button(this.id, this)
    }
}

expect fun Button.getClicked(): Boolean
expect fun Button.getHovering(): Boolean

// fits that of yakui_widgets::shorthand::button
fun UIBuilder.button(text: String, block: Button.() -> Unit = {}): Button {
    val btn = Button.styled(text).apply(block)
    instructions.add(btn.toInstruction())
    return btn
}