package com.dropbear.ui.widgets

import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.styling.Alignment

class Align(
    var align: Alignment,
): Widget()  {
    override lateinit var id: WidgetId

    companion object {
        fun center() = Align(align = Alignment.CENTER)
    }

    sealed class AlignmentInstruction: UIInstruction {
        data class Center(val id: WidgetId, val align: Align) : AlignmentInstruction()
    }

    fun toInstruction(): AlignmentInstruction.Center {
        return AlignmentInstruction.Center(this.id, this)
    }
}


fun UIBuilder.center(block: UIBuilder.() -> Unit = {}) {
    val align = Align.center()
    instructions.add(align.toInstruction())
    block()
}

fun UIBuilder.align(alignment: Alignment, block: UIBuilder.() -> Unit = {}) {
    val align = Align(alignment)
    instructions.add(align.toInstruction())
    block()
}