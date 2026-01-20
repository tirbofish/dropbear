package com.dropbear.ui

import com.dropbear.input.MouseButton
import com.dropbear.math.Vector2d
import com.dropbear.utils.ID

/**
 * The response given by an element that is drawn.
 */
class Response(
    val elementId: ID,
    private val ui: Ui
) {
    fun clicked(): Boolean = ui.wasClicked(elementId)
    fun clickedBy(button: MouseButton): Boolean = TODO("Not implemented yet")
    fun secondaryClicked(): Boolean = TODO("Not implemented yet")
    fun longTouched(): Boolean = TODO("Not implemented yet")
    fun middleClicked(): Boolean = TODO("Not implemented yet")
    fun doubleClicked(): Boolean = TODO("Not implemented yet")
    fun tripleClicked(): Boolean = TODO("Not implemented yet")
    fun doubleClickedBy(button: MouseButton): Boolean = TODO("Not implemented yet")
    fun tripleClickedBy(button: MouseButton): Boolean = TODO("Not implemented yet")
    fun clickedWithOpenInBackground(): Boolean = TODO("Not implemented yet")
    fun clickedElsewhere(): Boolean = TODO("Not implemented yet")
    fun enabled(): Boolean = TODO("Not implemented yet")
    fun hovered(): Boolean = TODO("Not implemented yet")
    fun hasFocus(): Boolean = TODO("Not implemented yet")
    fun gainedFocus(): Boolean = TODO("Not implemented yet")
    fun lostFocus() {}
    fun requestFocus() {}
    fun surrenderFocus() {}
    fun dragStarted(): Boolean = TODO("Not implemented yet")
    fun dragStartedBy(button: MouseButton): Boolean = TODO("Not implemented yet")
    fun dragged(): Boolean = TODO("Not implemented yet")
    fun draggedBy(button: MouseButton): Boolean = TODO("Not implemented yet")
    fun dragStopped(): Boolean = TODO("Not implemented yet")
    fun dragStoppedBy(button: MouseButton): Boolean = TODO("Not implemented yet")
    fun dragDelta(): Vector2d = TODO("Not implemented yet")
    fun totalDragDelta(): Vector2d = TODO("Not implemented yet")
    fun dragMotion(): Vector2d = TODO("Not implemented yet")

    // todo: payload

    fun interactPointerPos(): Vector2d? = TODO("Not implemented yet")
    fun hoverPos(): Vector2d = TODO("Not implemented yet")
    fun isPointerButtonDownOn(): Boolean = TODO("Not implemented yet")
    fun changed(): Boolean = TODO("Not implemented yet")
    fun markChanged() {}
    fun shouldClose(): Boolean = TODO("Not implemented yet")
    fun setClose() {}
    fun onHoverUi(ui: (Ui) -> Unit): Response = TODO("Not implemented yet")
    fun onDisabledHoverUi(ui: (Ui) -> Unit) : Response = TODO("Not implemented yet")
    fun onHoverUiAtPointer(ui: (Ui) -> Unit) : Response = TODO("Not implemented yet")
    fun showTooltipUi(ui: (Ui) -> Unit) {}
    fun showTooltipText(text: String) {}
    fun isTooltipOpen(): Boolean = TODO("Not implemented yet")
    fun onHoverTextAtPointer(text: String) {}
    fun onHoverText(text: String) {}
    fun onDisabledHoverText(text: String) {}
    fun highlight(): Boolean = TODO("Not implemented yet")
    fun interact(sense: Sense): Response = TODO("Not implemented yet")
}