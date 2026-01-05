package com.dropbear.lighting

/**
 * Types of lights available.
 */
enum class LightType {
    /**
     * A light that has a direction but no position, like sunlight.
     */
    Directional,

    /**
     * A light that emits from a single point in all directions, like a light bulb.
     */
    Point,

    /**
     * A light that emits from a point in a specific direction, like a torch.
     */
    Spot
}