package com.dropbear.physics

enum class ShapeCastStatus {
    /**
     * The shape-casting algorithm ran out of iterations before achieving convergence.
     *
     * The content of the `ShapeCastHit` will still be a conservative approximation of the actual result so
     * it is often fine to interpret this case as a success.
     *
     *  ### Note
     *  This documentation was taken from rapier3d directly.
     */
    OutOfIterations,

    /**
     * The shape-casting algorithm converged successfully.
     *
     *  ### Note
     *  This documentation was taken from rapier3d directly.
     */
    Converged,

    /**
     * Something went wrong during the shape-casting, likely due to numerical instabilities.
     *
     * The content of the `ShapeCastHit` will still be a conservative approximation of the actual result so
     * it is often fine to interpret this case as a success.
     *
     *  ### Note
     *  This documentation was taken from rapier3d directly.
     */
    Failed,

    /**
     *  The two shape already overlap, or are separated by a distance smaller than
     *  `ShapeCastOptions::target_distance` at the time 0.
     *
     *  The witness points and normals provided by the `ShapeCastHit` will have unreliable values unless
     *  `ShapeCastOptions::compute_impact_geometry_on_penetration` was set to `true` when calling
     *  the time-of-impact function.
     *
     *  ### Note
     *  This documentation was taken from rapier3d directly.
     */
    PenetratingOrWithinTargetDist,
}