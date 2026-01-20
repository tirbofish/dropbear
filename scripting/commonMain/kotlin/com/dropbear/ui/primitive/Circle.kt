package com.dropbear.ui.primitive

import com.dropbear.math.Vector2d
import com.dropbear.ui.Response
import com.dropbear.ui.Ui
import com.dropbear.ui.Widget
import com.dropbear.ui.pushCircle
import com.dropbear.utils.ID

/**
 * Creates a standard circle.
 *
 * @property center The center of the circle.
 * @property radius The distance from the perimeter of the circle to the center/the radius.
 */
class Circle(
    val id: ID,
    var center: Vector2d,
    var radius: Double,
): Widget(id) {
    override fun draw(ui: Ui): Response {
        ui.pushCircle(this)
        return Response(id, ui)
    }
}