package com.dropbear.ui.widgets

import com.dropbear.ui.Response
import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.styling.Padding
import com.dropbear.ui.styling.TextStyle

class Text(
    var text: String,
    var style: TextStyle = TextStyle(),
    var padding: Padding,
    id: WidgetId = WidgetId(text.hashCode().toLong()),
) : Widget() {
    override var id: WidgetId

    init {
        this.id = id
    }

    val response: Response
        get() = getResponse()

    companion object {
        fun withStyle(text: String, style: TextStyle, id: String = text): Text {
            val text = Text(text, style, padding = Padding.zero())
            text.id = WidgetId(id.hashCode().toLong())
            return text
        }

        fun label(text: String, id: String = text): Text {
            val text = Text(text, TextStyle(), Padding.all(8.0))
            text.id = WidgetId(id.hashCode().toLong())
            return text
        }
    }

    sealed class TextInstruction: UIInstruction {
        data class Text(val id: WidgetId, val text: com.dropbear.ui.widgets.Text) : TextInstruction()
    }

    override fun toInstruction(): List<UIInstruction> {
        return listOf(TextInstruction.Text(this.id, this))
    }
}

fun UIBuilder.label(text: String, block: Text.() -> Unit = {}): Text {
    val style = TextStyle()
    val text = Text(
        text = text,
        style = style,
        padding = Padding.zero()
    ).apply(block)
    text.toInstruction().forEach {
        instructions.add(it)
    }
    return text
}

fun UIBuilder.text(size: Double, text: String, block: Text.() -> Unit = {}): Text {
    val style = TextStyle()
    style.fontSize = size
    val text = Text(
        text = text,
        style = style,
        padding = Padding.zero()
    ).apply(block)
    text.toInstruction().forEach {
        instructions.add(it)
    }
    return text
}

expect fun Text.getResponse(): Response