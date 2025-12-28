package com.dropbear.physics

/**
 * How this entity behaves in the physics simulation.
 *
 * This intentionally mirrors Rapier's rigid-body types, but stays engine-owned and serializable.
 */
enum class RigidBodyMode {
    /**
     * A fully simulated body affected by forces and contacts.
     */
    Dynamic,

    /**
     * An immovable body.
     */
    Fixed,

    /**
     * A kinematic body controlled by setting its next position.
     */
    KinematicPosition,

    /**
     * A kinematic body controlled by setting its velocities.
     */
    KinematicVelocity,
}