package com.dropbear.ui.widgets

import com.dropbear.math.Vector2d
import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.styling.Anchor

class Column(
	override var id: WidgetId,
	var anchor: Anchor = Anchor.TopLeft,
	var position: Vector2d = Vector2d.zero(),
	var spacing: Double = 8.0,
): Widget() {
	sealed class ColumnInstruction: UIInstruction {
		data class StartColumnBlock(val id: WidgetId, val column: com.dropbear.ui.widgets.Column) : ColumnInstruction()
		data class EndColumnBlock(val id: WidgetId) : ColumnInstruction()
	}

	override fun toInstruction(): List<UIInstruction> {
		return listOf(ColumnInstruction.StartColumnBlock(id, this), ColumnInstruction.EndColumnBlock(id))
	}

	fun toContaineredInstructions(children: List<UIInstruction>): List<UIInstruction> {
		return listOf(ColumnInstruction.StartColumnBlock(id, this)) +
			children +
			ColumnInstruction.EndColumnBlock(id)
	}

	fun startInstruction(): UIInstruction = ColumnInstruction.StartColumnBlock(id, this)

	fun endInstruction(): UIInstruction = ColumnInstruction.EndColumnBlock(id)

	fun at(position: Vector2d): Column {
		this.position = position
		return this
	}

	fun spacing(spacing: Double): Column {
		this.spacing = spacing
		return this
	}

	fun withAnchor(anchor: Anchor): Column {
		this.anchor = anchor
		return this
	}
}

fun UIBuilder.column(id: WidgetId = WidgetId(generateId().toLong()), block: Column.() -> Unit = {}): Column {
	val column = Column(id = id).apply(block)
	column.toInstruction().forEach { instructions.add(it) }
	return column
}

fun UIBuilder.column(id: WidgetId, content: UIBuilder.(Column) -> Unit) {
	column(id, columnBlock = {}, content = content)
}

fun UIBuilder.column(
	id: WidgetId,
	columnBlock: Column.() -> Unit = {},
	content: UIBuilder.(Column) -> Unit
) {
	val column = Column(id = id).apply(columnBlock)
	instructions.add(column.startInstruction())
	this.content(column)
	instructions.add(column.endInstruction())
}