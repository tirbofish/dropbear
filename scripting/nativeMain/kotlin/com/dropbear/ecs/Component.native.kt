package com.dropbear.ecs

import com.dropbear.EntityId


actual fun registerKotlinComponentType(
    fqcn: String,
    typeName: String?,
    category: String?,
    description: String?,
) = Unit