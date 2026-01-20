package com.dropbear.ui

/**
 * What sort of interaction is a widget sensitive to?
 */
class Sense(val bit: Int) {
    companion object {
        const val HOVER = 0

        /**
         * Buttons, sliders, windows, …
         */
        const val CLICK = 1 shl 0

        /**
         * Sliders, windows, scroll bars, scroll areas, …
         */
        const val DRAG = 1 shl 1

        /**
         * This widget wants focus.
         *
         * Anything interactive + labels that can be focused for the benefit of screen readers.
         */
        const val FOCUSABLE = 1 shl 2

        /**
         * Senses no clicks or drags. Only senses mouse hover.
         */
        fun hover(): Sense {
            return Sense(HOVER)
        }

        /**
         * Senses no clicks or drags, but can be focused with the keyboard.
         *
         * Used for labels that can be focused for the benefit of screen readers.
         */
        fun focusableNonInteractive(): Sense {
            return Sense(FOCUSABLE)
        }

        /**
         * Sense clicks and hover, but not drags.
         */
        fun click(): Sense {
            return Sense(CLICK or FOCUSABLE)
        }

        /**
         * Sense drags and hover, but not clicks.
         */
        fun drag(): Sense {
            return Sense(DRAG or FOCUSABLE)
        }

        /**
         * Sense both clicks, drags and hover (e.g. a slider or window).
         *
         * Note that this will introduce a latency when dragging,
         * because when the user starts a press egui can't know if this is the start
         * of a click or a drag, and it won't know until the cursor has
         * either moved a certain distance, or the user has released the mouse button.
         */
        fun clickAndDrag(): Sense {
            return Sense(CLICK or FOCUSABLE or DRAG)
        }
    }
}