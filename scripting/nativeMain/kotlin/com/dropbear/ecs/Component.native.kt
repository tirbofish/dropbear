@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.ecs

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import kotlinx.cinterop.*

internal actual fun hasKotlinComponent(entityId: EntityId, fqcn: String): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    dropbear_kotlin_component_exists(world, entityId.raw.toULong(), fqcn)
}

internal actual fun registerKotlinComponentType(
    fqcn: String,
    typeName: String?,
    category: String?,
    description: String?,
) {
    dropbear_register_kotlin_component(fqcn, typeName, category, description)
}
