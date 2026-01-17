package com.dropbear.utils

/**
 * Used to show the progress of something.
 *
 * @param current The current index of entities being loaded.
 * @param total The total amount of entities being loaded
 * @param message The message/the current entity being loaded. Can be null if nothing.
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

    /**
     * Returns the object as a percentage.
     *
     * Its just ([current]/[total]) * 100.
     */
    fun percentage(): Double {
        return (current / total) * 100
    }

    override fun toString(): String {
        return "${(current / total) * 100}% ; ${message ?: ""}"
    }
}