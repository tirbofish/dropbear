package io.github.tirbofish.surfaceNets

import com.dropbear.DropbearEngine
import com.dropbear.EntityId

internal actual fun surfaceNetsExistsForEntity(entityId: EntityId): Boolean {
    return SurfaceNetsNative.surfaceNetsExistsForEntity(
        DropbearEngine.native.worldHandle,
        entityId.raw
    )
}
