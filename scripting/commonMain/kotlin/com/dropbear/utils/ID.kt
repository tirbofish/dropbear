package com.dropbear.utils

/**
 * Describes all classes that can be supplied as a form of identification, such as that of a hashcode
 * or a number.
 *
 * This is different to [com.dropbear.asset.Handle] as a handle is used for assets, while an [ID] is used
 * for identifying something.
 *
 * @param id The raw id value
 */
open class ID(id: Long) {
    companion object {
        fun fromString(id: String): ID {
            return ID(hashCode().toLong())
        }
    }

    // it has to be like this EntityId already took `raw`.
    private val rawId: Long = id

    fun getId(): Long = rawId

    override fun toString(): String {
        return "ID(raw=${getId()})"
    }
}

/**
 * Converts a [String] into an [ID], allowing to be used for UI.
 */
fun String.asId(): ID {
    return ID(hashCode().toLong())
}