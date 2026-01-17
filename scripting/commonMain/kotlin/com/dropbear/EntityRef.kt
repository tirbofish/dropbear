package com.dropbear

import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.physics.RigidBody

/**
 * A reference to an ECS Entity stored inside the dropbear engine.
 *
 * The dropbear engine prefers careful mutability, which is why a reference is passed (as a handle) instead
 * of its full information. Also conserves memory.
 *
 * The ECS system the dropbear engine uses is `hecs` ECS, which is a Rust crate that has blazing fast
 * querying systems. The id passed is just a primitive integer value that points to the entity in the world.
 *
 * @property id The unique identifier of the entity as set by `hecs::World`. This value changes dynamically during different
 *              playthroughs, so it is recommended not to store this value.
 */
class EntityRef(val id: EntityId = EntityId(0L)) {
    companion object {
        /**
         * Creates an [EntityRef] from an existing [EntityId].
         *
         * This function checks the validity of an [EntityId] by querying it for a [com.dropbear.components.Label]
         * entity. If it does not exist, it means that there is no entity (as all entities contain a Label component).
         */
        fun fromEntityId(id: EntityId): EntityRef {
            if (getEntityLabel(id).isNotBlank()) {
                return EntityRef(id)
            } else {
                // no need for this, its just to ensure the typesafety.
                throw IllegalArgumentException("Entity $id does not exist.")
            }
        }
    }

    /**
     * The `Label` component (the entity name).
     *
     * All entities have a `Label` component. If one does not, it is considered a bug in the engine or *you* did something
     * to break this. Anyhow, it will throw an [Exception].
     */
    val label: String
        get() = getEntityLabel(id)

    override fun toString(): String {
        return "EntityRef(id=$id)"
    }

    /**
     * Fetches a component that is attached to an entity.
     *
     * # Example
     * ```kotlin
     * val rb = engine.getComponent(RigidBody) ?: return
     * ```
     *
     * @param type The component being queried.
     * @return The type being queried, or `null` if not attached.
     */
    fun <T : Component> getComponent(type: ComponentType<T>): T? {
        return type.get(id)
    }

    /**
     * Fetches all direct children available to that entity. It does not go any deeper than that level.
     *
     * It will return `null` if there was an error, or an empty array if no children have been found.
     *
     * # Example
     * ```
     * |- cat
     * |    |- wizard_hat
     * |    |    |- pom_pom
     * ```
     *
     * By running [getChildren] on `cat`, it will return `[ wizard_hat ]`, not `pom_pom`.
     */
    fun getChildren(): Array<EntityRef>? {
        return getChildren(id)
    }

    /**
     * Fetches a direct child by a specific label.
     *
     * Returns `null` if an error occurred or no child exists, otherwise the entity.
     */
    fun getChildByLabel(label: String): EntityRef? {
        return getChildByLabel(id, label)
    }

    /**
     * Fetches the parent of this entity.
     *
     * Returns `null` if no parent exists. If it exists, it will return the [EntityRef] of that parent.
     *
     * # Note
     * You will see in the editor something like this:
     * ```
     * Scene_name
     * |- cat
     * |- bat
     * ```
     *
     * Calling [getParent] on `cat` will return `null`, as the Scene is not an entity.
     */
    fun getParent(): EntityRef? {
        return getParent(id)
    }
}

internal expect fun EntityRef.Companion.getEntityLabel(entity: EntityId): String
internal expect fun EntityRef.getChildren(entityId: EntityId): Array<EntityRef>?
internal expect fun EntityRef.getChildByLabel(entityId: EntityId, label: String): EntityRef?
internal expect fun EntityRef.getParent(entityId: EntityId): EntityRef?