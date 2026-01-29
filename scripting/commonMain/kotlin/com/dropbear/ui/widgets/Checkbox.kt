package com.dropbear.ui.widgets

import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId

class Checkbox(
    override val id: WidgetId,
    initiallyChecked: Boolean,
) : Widget() {
    private var cachedChecked: Boolean = initiallyChecked

    /**
     * To set a new checkbox value, make it so the class is different next frame.
     */
    var checked: Boolean
        get() {
            if (hasCheckedState()) {
                cachedChecked = getChecked()
            }
            return cachedChecked
        }
        private set(value) {
            cachedChecked = value
        }

    init {
        checked = initiallyChecked
    }

    sealed class CheckboxInstruction: UIInstruction {
        data class Checkbox(val id: WidgetId, val checked: Boolean) : CheckboxInstruction()
    }

    override fun toInstruction(): List<UIInstruction> {
        return listOf(CheckboxInstruction.Checkbox(id, checked))
    }
}

expect fun Checkbox.getChecked(): Boolean
expect fun Checkbox.hasCheckedState(): Boolean

fun UIBuilder.checkbox(checked: Boolean, id: WidgetId, block: Checkbox.() -> Unit = {}): Checkbox {
    val cb = Checkbox(id, checked).apply(block)
    cb.toInstruction().forEach { instruction -> instructions.add(instruction) }
    return cb
}