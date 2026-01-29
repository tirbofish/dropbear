package com.dropbear.ui.widgets

import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.styling.Alignment

class Align(
    override var id: WidgetId,
    var align: Alignment,
    var widgets: MutableList<Widget> = mutableListOf(),
): Widget() {
    companion object {
        fun center(id: WidgetId) = Align(align = Alignment.CENTER, id = id)
    }

    sealed class AlignmentInstruction: UIInstruction {
        data class StartAlignmentBlock(val id: WidgetId, val align: Align) : AlignmentInstruction()
        data class EndAlignmentBlock(val id: WidgetId) : AlignmentInstruction()
    }

    override fun toInstruction(): List<UIInstruction> {
        val instructions = mutableListOf<UIInstruction>()
        instructions.add(AlignmentInstruction.StartAlignmentBlock(id, this))
        widgets.forEach { widget ->
            widget.toInstruction().forEach { instruction ->
                instructions.add(instruction)
            }
        }
        instructions.add(AlignmentInstruction.EndAlignmentBlock(id))
        return instructions.toList()
    }
}

fun UIBuilder.center(id: WidgetId, block: UIBuilder.() -> Unit = {}) {
    val align = Align.center(id)
    instructions.add(Align.AlignmentInstruction.StartAlignmentBlock(align.id, align))
    block(this)
    instructions.add(Align.AlignmentInstruction.EndAlignmentBlock(align.id))
}

fun UIBuilder.align(alignment: Alignment, id: WidgetId, block: UIBuilder.() -> Unit = {}) {
    val align = Align(align = alignment, id = id)
    instructions.add(Align.AlignmentInstruction.StartAlignmentBlock(align.id, align))
    block(this)
    instructions.add(Align.AlignmentInstruction.EndAlignmentBlock(align.id))
}