package com.dropbear.ui.primitive

import com.dropbear.math.Vector2d
import com.dropbear.ui.Response
import com.dropbear.ui.StrokeKind
import com.dropbear.ui.Ui
import com.dropbear.ui.Widget
import com.dropbear.ui.pushRect
import com.dropbear.utils.Colour
import com.dropbear.utils.ID

/**
 * @property initial The top left position/coordinate of the rectangle.
 * @property width The width of the rectangle
 * @property height The height of the rectangle
 * @property cornerRadius The radius of the corner. The higher the value, the more smoothened out the rect is
 * @property fillColour The colour of inside the rectangle. By default, it is [Colour.WHITE]
 * @property stroke The thickness/width of the stroke.
 * @property strokeColour The colour of the stroke. By default, it is [Colour.BLACK].
 * @property strokeKind Describes the stroke of a shape
 */
class Rectangle(
    val id: ID,
    var initial: Vector2d,
    var width: Double,
    var height: Double,
    var cornerRadius: Double = 0.0,
    var fillColour: Colour = Colour.WHITE,
    var stroke: Double = 0.0,
    var strokeColour: Colour = Colour.BLACK,
    var strokeKind: StrokeKind = StrokeKind.Middle,
): Widget(id) {
    override fun draw(ui: Ui): Response {
        ui.pushRect(this)
        return Response(id, ui)
    }
}