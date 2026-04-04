package io.github.tirbofish.surfaceNets

import com.dropbear.EntityId
import com.dropbear.ecs.ComponentType
import com.dropbear.ecs.ExternalComponent

class SurfaceNets(val id: EntityId) : ExternalComponent(
    fullyQualifiedTypeName = "surface_nets_plugin::component::SurfaceNets",
) {
    companion object : ComponentType<SurfaceNets> {
        override fun get(entityId: EntityId): SurfaceNets? {
            return if (surfaceNetsExistsForEntity(entityId)) SurfaceNets(entityId) else null
        }
    }
}

internal expect fun surfaceNetsExistsForEntity(entityId: EntityId): Boolean
