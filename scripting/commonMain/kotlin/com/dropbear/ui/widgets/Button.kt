package com.dropbear.ui.widgets

import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.styling.Alignment
import com.dropbear.ui.styling.Border
import com.dropbear.ui.styling.BorderRadius
import com.dropbear.ui.styling.DynamicButtonStyle
import com.dropbear.ui.styling.Padding
import com.dropbear.ui.styling.TextStyle
import com.dropbear.ui.styling.fonts.TextAlignment
import com.dropbear.utils.Colour
import com.dropbear.utils.ID

class Button(
    var text: String,
    var alignment: Alignment,
    var padding: Padding,
    var borderRadius: BorderRadius,
    var style: DynamicButtonStyle,
    var hoverStyle: DynamicButtonStyle,
    var downStyle: DynamicButtonStyle,
    id: WidgetId = WidgetId(text.hashCode().toLong()),
): Widget() {
    override var id: WidgetId

    init {
        this.id = id
    }

    val clicked: Boolean
        get() = getClicked()

    val hovering: Boolean
        get() = getHovering()

    companion object {
        fun styled(text: String, id: String = text) : Button {
            val style = DynamicButtonStyle(
                fill = Colour.BACKGROUND_3,
                text = TextStyle(
                    colour = Colour.WHITE.adjust(0.8),
                    align = TextAlignment.Center
                ),
                border = Border(
                    colour = Colour.BACKGROUND_1,
                    width = 1.0
                )
            )

            val hoverStyle = DynamicButtonStyle(
                fill = Colour.BACKGROUND_3.adjust(1.2),
                border = Border(Colour.WHITE.adjust(0.75), 1.0),
            )

            val downStyle = DynamicButtonStyle(
                fill = Colour.BACKGROUND_3.adjust(0.8),
                border = Border(Colour.WHITE, 1.0)
            )

            val result = Button(
                text = text,
                alignment = Alignment.CENTER,
                padding = Padding.balanced(20.0, 10.0),
                borderRadius = BorderRadius.uniform(6.0),
                style = style,
                hoverStyle = hoverStyle,
                downStyle = downStyle,
            )

            result.id = WidgetId(id.hashCode().toLong())

            return result
        }

        fun unstyled(text: String, id: String = text) : Button {
            val result = Button(
                text = text,
                alignment = Alignment.CENTER,
                padding = Padding.zero(),
                borderRadius = BorderRadius(),
                style = DynamicButtonStyle(),
                hoverStyle = DynamicButtonStyle(),
                downStyle = DynamicButtonStyle(),
            )

            result.id = WidgetId(id.hashCode().toLong())

            return result
        }
    }

    sealed class ButtonInstruction: UIInstruction {
        data class Button(val id: WidgetId, val button: com.dropbear.ui.widgets.Button) : ButtonInstruction()
    }

    override fun toInstruction(): List<UIInstruction> {
        return listOf(ButtonInstruction.Button(this.id, this))
    }
}

expect fun Button.getClicked(): Boolean
expect fun Button.getHovering(): Boolean

// fits that of yakui_widgets::shorthand::button
fun UIBuilder.button(text: String, block: Button.() -> Unit = {}): Button {
    val btn = Button.styled(text).apply(block)
    btn.toInstruction().forEach {
        instructions.add(it)
    }
    return btn
}