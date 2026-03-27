package com.dropbear.ecs

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.EntityRef
import com.dropbear.ui.UIInstructionSet

/**
 * States a Kotlin-defined ECS component.
 *
 * Annotate subclasses with [@EcsComponent][EcsComponent] so that `magna-carta` can discover them
 * at build time and emit a `ComponentManager` that calls [register] for every discovered type.
 *
 * @param fullyQualifiedTypeName The fully qualified class name used as the stable key inside the
 *   Rust ComponentRegistry (e.g. `"com.example.PlayerHealth"`). Prefer using the literal string
 *   directly so the value survives code shrinking / obfuscation.
 * @param typeName Human-readable short name shown in the editor (e.g. `"PlayerHealth"`).
 * @param category Optional grouping shown in the "Add Component" picker (e.g. `"Gameplay"`).
 * @param description Optional one-line description shown as a tooltip in the editor.
 */
abstract class NativeComponent(
    val fullyQualifiedTypeName: String,
    val typeName: String,
    val category: String? = null,
    val description: String? = null,
): Component() {
    /**
     * The entity currently being updated.
     *
     * Set by the engine immediately before [updateComponent] is called and cleared
     * immediately after. It is `null` outside of an active update call.
     */
    var currentEntity: EntityRef? = null
        private set

    /**
     * Registers this component type with the engine's ComponentRegistry.
     *
     * This is called once per type at startup by the generated [ComponentManager].
     */
    fun register() {
        registerKotlinComponentType(fullyQualifiedTypeName, typeName, category, description)
    }

    /** Internal: sets [currentEntity] before calling [updateComponent]. */
    fun setCurrentEntity(entity: Long) {
        currentEntity = EntityRef(EntityId(entity))
    }

    /** Internal: clears [currentEntity] after [updateComponent] returns. */
    fun clearCurrentEntity() {
        currentEntity = null
    }

    /**
     * Renders the inspector UI for this component in the editor.
     *
     * Called by the editor for every entity that has this component type attached.
     */
    abstract fun inspect(engine: DropbearEngine): UIInstructionSet?

    /**
     * Called once per frame for every entity that holds this component type.
     *
     * [currentEntity] is set to the entity being processed for the duration of this call.
     *
     * @param engine  The engine facade for queries and mutations.
     * @param deltaTime Seconds elapsed since the last frame.
     */
    abstract fun updateComponent(engine: DropbearEngine, deltaTime: Double)
}