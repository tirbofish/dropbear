package com.dropbear.ui

/**
 * Describes how the stroke of a shape should be painted.
 */
enum class StrokeKind {
    /**
     * The stroke should be painted entirely inside the shape
     */
    Inside,

    /**
     * The stroke should be painted right on the edge of the shape, half inside and half outside.
     */
    Middle,

    /**
     * The stroke should be painted entirely outside the shape
     */
    Outside,
}