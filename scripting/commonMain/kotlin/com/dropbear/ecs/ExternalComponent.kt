package com.dropbear.ecs

/**
 * Proxy for an ECS component that is defined outside Kotlin, typically in a Rust crate or
 * another native shared library, and registered with the engine's ComponentRegistry under a
 * known [fullyQualifiedTypeName].
 *
 * Use this when you need to query whether an entity carries a non-Kotlin component, without
 * owning or duplicating its data on the Kotlin side.
 *
 * @param fullyQualifiedTypeName The fully qualified type name that the external component was
 *   registered under in the ComponentRegistry (e.g. `"eucalyptus_core::components::RigidBody"`).
 */
abstract class ExternalComponent(
    val fullyQualifiedTypeName: String,
): Component() {
}