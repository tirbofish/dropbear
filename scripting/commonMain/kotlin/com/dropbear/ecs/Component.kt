package com.dropbear.ecs

expect fun registerKotlinComponentType(
    fqcn: String,
    typeName: String?,
    category: String?,
    description: String?,
)

/**
 * The base class of any component.
 *
 * Extend [NativeComponent] or [ExternalComponent], not this class directly.
 */
sealed class Component