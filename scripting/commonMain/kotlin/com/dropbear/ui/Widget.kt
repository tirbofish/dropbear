package com.dropbear.ui

import com.dropbear.utils.ID

/**
 * An abstract class all drawable objects must inherit
 */
abstract class Widget(id: ID) {
    /**
     * Draws the object.
     */
    abstract fun draw(ui: Ui) : Response
}