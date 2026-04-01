package com.dropbear.logging

/**
 * Extension function to log and return a value if the value is null.
 *
 * # Example
 * ```
 * val room1Sensor = engine
 *     .getEntity("room1_sensor")
 *     ?.getComponent(ColliderGroup)
 *     ?.getColliders()
 *     .orLogAndReturn("room1_sensor missing") { return } // returns if the value is null
 * ```
 */
inline fun <T> T?.orLogAndReturn(msg: String, returnBlock: () -> Nothing): T {
    return this ?: run {
        Logger.warn(msg)
        returnBlock()
    }
}