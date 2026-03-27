package com.dropbear.ecs

import com.dropbear.EntityId


internal actual fun registerKotlinComponentType(
    fqcn: String,
    typeName: String?,
    category: String?,
    description: String?,
) = Unit