package com.dropbear.ui.widgets

import com.dropbear.math.Vector2d
import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.buildUI
import com.dropbear.utils.Colour

class ColouredBox(
    override val id: WidgetId,
    var colour: Colour = Colour.WHITE,
    var minSize: Vector2d = Vector2d.zero(),
) : Widget() {
    var widgets: MutableList<UIInstruction> = mutableListOf()

    companion object {
        fun sized(id: WidgetId, colour: Colour, size: Vector2d): ColouredBox {
            return ColouredBox(id, colour, size)
        }

        fun container(id: WidgetId, colour: Colour): ColouredBox {
            return ColouredBox(id, colour, Vector2d.zero())
        }
    }

    sealed class ColouredBoxInstruction: UIInstruction {
        data class StartColouredBoxInstruction(val id: WidgetId, val box: ColouredBox): ColouredBoxInstruction()
        data class EndColouredBoxInstruction(val id: WidgetId): ColouredBoxInstruction()
    }

    override fun toInstruction(): List<UIInstruction> {
        val list = mutableListOf<UIInstruction>()
        list.add(ColouredBoxInstruction.StartColouredBoxInstruction(this.id, this))
        list.addAll(widgets)
        list.add(ColouredBoxInstruction.EndColouredBoxInstruction(this.id))
        return list.toList()
    }

    fun addChild(widget: Widget) {
        widgets.addAll(widget.toInstruction())
    }
}

fun UIBuilder.colouredBox(id: WidgetId, colour: Colour, minSize: Vector2d, block: ColouredBox.() -> Unit = {}): ColouredBox {
    val box = ColouredBox(id, colour, minSize).apply(block)
    instructions.addAll(box.toInstruction())
    return box
}

fun UIBuilder.colouredBoxContainer(id: WidgetId, colour: Colour, children: UIBuilder.() -> Unit): ColouredBox {
    val box = ColouredBox(id = id, colour = colour)
    val newUI = UIBuilder()
    children(newUI) // execute
    box.widgets.addAll(newUI.build())
    instructions.addAll(box.toInstruction())
    return box
}

fun dummy() {
    val instructions = buildUI {
        colouredBox(WidgetId("something"), Colour.TRANSPARENT, Vector2d.zero())

        colouredBoxContainer(WidgetId("something"), Colour.TRANSPARENT) {

        }
    }
}