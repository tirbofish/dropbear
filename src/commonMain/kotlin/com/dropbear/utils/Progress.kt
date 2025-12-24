package com.dropbear.utils

/**
 * Used to show the progress of something.
 */
class Progress(var current: Double, var total: Double, var message: String?) {
    companion object {
        /**
         * Returns absolutely no progress. Typically used for conveying an error
         */
        fun nothing(): Progress {
            return Progress(0.0, 0.0, null)
        }

        /**
         * Returns absolutely no progress. Typically used for conveying an error.
         *
         * An optional message can be provided if available. Is useful for checking for any errors.
         */
        fun nothing(message: String): Progress {
            return Progress(0.0, 0.0, message)
        }
    }

    fun percentage(): Double {
        return (current / total) * 100
    }

    override fun toString(): String {
        return "${(current / total) * 100}% ; ${message ?: ""}"
    }
}