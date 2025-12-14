package com.dropbear.utils

/**
 * Used to show the progress of something.
 */
class Progress(var current: Double, var total: Double, var message: String?) {
    fun percentage(): Double {
        return current / total
    }

    override fun toString(): String {
        return "${(current / total) * 100}% ; ${message ?: ""}"
    }
}