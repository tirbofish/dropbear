package com.dropbear.ui.widgets

import com.dropbear.math.Vector2d
import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.styling.Anchor

class Row(
	override var id: WidgetId,
	var anchor: Anchor = Anchor.TopLeft,
	var position: Vector2d = Vector2d.zero(),
	var spacing: Double = 8.0,
): Widget() {
	sealed class RowInstruction: UIInstruction {
		data class StartRowBlock(val id: WidgetId, val row: com.dropbear.ui.widgets.Row) : RowInstruction()
		data class EndRowBlock(val id: WidgetId) : RowInstruction()
	}

	override fun toInstruction(): List<UIInstruction> {
		return listOf(RowInstruction.StartRowBlock(id, this), RowInstruction.EndRowBlock(id))
	}

	fun toContaineredInstructions(children: List<UIInstruction>): List<UIInstruction> {
		return listOf(RowInstruction.StartRowBlock(id, this)) +
			children +
			RowInstruction.EndRowBlock(id)
	}

	fun startInstruction(): UIInstruction = RowInstruction.StartRowBlock(id, this)

	fun endInstruction(): UIInstruction = RowInstruction.EndRowBlock(id)

	fun at(position: Vector2d): Row {
		this.position = position
		return this
	}

	fun spacing(spacing: Double): Row {
		this.spacing = spacing
		return this
	}

	fun withAnchor(anchor: Anchor): Row {
		this.anchor = anchor
		return this
	}
}

fun UIBuilder.row(id: WidgetId = WidgetId(generateId().toLong()), block: Row.() -> Unit = {}): Row {
	val row = Row(id = id).apply(block)
	row.toInstruction().forEach { instructions.add(it) }
	return row
}

fun UIBuilder.row(id: WidgetId, content: UIBuilder.(Row) -> Unit) {
	row(id, rowBlock = {}, content = content)
}

fun UIBuilder.row(
	id: WidgetId,
	rowBlock: Row.() -> Unit = {},
	content: UIBuilder.(Row) -> Unit
) {
	val row = Row(id = id).apply(rowBlock)
	instructions.add(row.startInstruction())
	this.content(row)
	instructions.add(row.endInstruction())
}