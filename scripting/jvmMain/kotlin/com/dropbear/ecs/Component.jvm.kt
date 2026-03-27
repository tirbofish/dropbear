package com.dropbear.ecs

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.components.ComponentNative

actual fun registerKotlinComponentType(
    fqcn: String,
    typeName: String?,
    category: String?,
    description: String?,
) = ComponentNative.registerKotlinComponent(
    DropbearEngine.native.worldHandle,
    fqcn,
    typeName ?: "",
    category ?: "",
    description ?: "",
)
