package com.dropbear.ui.widgets

import com.dropbear.asset.Handle
import com.dropbear.asset.Texture
import com.dropbear.math.Angle
import com.dropbear.math.Vector2d
import com.dropbear.ui.Response
import com.dropbear.ui.UIBuilder
import com.dropbear.ui.UIInstruction
import com.dropbear.ui.Widget
import com.dropbear.ui.WidgetId
import com.dropbear.ui.styling.Anchor
import com.dropbear.ui.styling.Border
import com.dropbear.ui.styling.Fill
import com.dropbear.utils.Colour

class Rectangle(
    override var id: WidgetId,
    var anchor: Anchor = Anchor.TopLeft,
    var position: Vector2d = Vector2d.zero(),
    var size: Vector2d = Vector2d(64.0, 128.0),
    var texture: Handle<Texture>? = null,
    var rotation: Angle = Angle.ZERO,
    var uv: List<Vector2d> = listOf(Vector2d.zero(), Vector2d(1.0, 0.0), Vector2d.one(), Vector2d(0.0, 1.0)),
    var fill: Fill = Fill(Colour.WHITE),
    var border: Border? = null
): Widget() {
    init {
        require(uv.size == 4) { "Rectangle uv must have 4 coordinates." }
    }
    
    val response: Response
        get() = getResponse()
    
    sealed class RectangleInstruction: UIInstruction {
        data class Rectangle(val id: WidgetId, val rect: com.dropbear.ui.widgets.Rectangle) : RectangleInstruction()
        data class StartRectangleBlock(val id: WidgetId, val rect: com.dropbear.ui.widgets.Rectangle) : RectangleInstruction()
        data class EndRectangleBlock(val id: WidgetId) : RectangleInstruction()
    }

    override fun toInstruction(): List<UIInstruction> {
        return listOf(RectangleInstruction.Rectangle(id, this))
    }

    fun toContaineredInstructions(children: List<UIInstruction>): List<UIInstruction> {
        return listOf(RectangleInstruction.StartRectangleBlock(id, this)) +
            children +
            RectangleInstruction.EndRectangleBlock(id)
    }

    fun startInstruction(): UIInstruction = RectangleInstruction.StartRectangleBlock(id, this)

    fun endInstruction(): UIInstruction = RectangleInstruction.EndRectangleBlock(id)

    fun with(position: Vector2d, size: Vector2d): Rectangle {
        this.position = position
        this.size = size
        return this
    }

    fun withAnchor(anchor: Anchor): Rectangle {
        this.anchor = anchor
        return this
    }

    fun at(position: Vector2d): Rectangle {
        this.position = position
        return this
    }

    fun size(size: Vector2d): Rectangle {
        this.size = size
        return this
    }

    fun fill(fill: Fill): Rectangle {
        this.fill = fill
        return this
    }

    fun border(border: Border?): Rectangle {
        this.border = border
        return this
    }

    fun rotate(angle: Angle): Rectangle {
        this.rotation = angle
        return this
    }

    fun texture(texture: Handle<Texture>?): Rectangle {
        this.texture = texture
        return this
    }

    fun uv(coords: List<Vector2d>): Rectangle {
        require(coords.size == 4) { "Rectangle uv must have 4 coordinates." }
        this.uv = coords
        return this
    }
}

fun UIBuilder.rectangle(id: WidgetId = WidgetId(generateId().toLong()), block: Rectangle.() -> Unit = {}): Rectangle {
    val rect = Rectangle(id = id).apply(block)
    rect.toInstruction().forEach { instructions.add(it) }
    return rect
}

fun UIBuilder.container(id: WidgetId, block: UIBuilder.(Rectangle) -> Unit) {
    container(id, rectBlock = {}, content = block)
}

fun UIBuilder.container(
    id: WidgetId,
    rectBlock: Rectangle.() -> Unit = {},
    content: UIBuilder.(Rectangle) -> Unit
) {
    val rect = Rectangle(id = id).apply(rectBlock)
    instructions.add(rect.startInstruction())
    this.content(rect)
    instructions.add(rect.endInstruction())
}

expect fun Rectangle.getResponse(): Response