package com.dropbear

/**
 * A class that contains the basic information of a system. 
 * 
 * The dropbear engine follows an ECS paradigm, with logic being
 * provided as Systems.
 */
open class System {
    /**
     * The current entity that is being run in the system. Most often than not, it
     * will have an [EntityRef] attached (because all Script components must be part
     * of an entity).
     */
    var currentEntity: EntityRef? = null
        private set

    private var engineRef: DropbearEngine? = null

    /**
     * This function is called when the script module is initialised.
     *
     * It is only called once during scene execution. If you re-switch back
     * to this scene after running the class, it will be run again.
     */
    open fun load(engine: DropbearEngine) {}

    /**
     * This function is called for each update.
     *
     * It is run once for each frame, and for every frame. Since this is synced to the frame rate, using
     * the [deltaTime] variable can aid you in creating uniform player speeds (or something like that).
     *
     * @param deltaTime - This specifies the time elapsed since the last update.
     */
    open fun update(engine: DropbearEngine, deltaTime: Float) {}

    /**
     * This function is called for each update that is related to physics.
     *
     * It can be run 0, 1, 2 or more times per frame. Updating physics is done at a constant
     * rate/tick (at roughly 50Hz or 0.02), which is why it is not ran as often as a standard [update].
     *
     * @param deltaTime - This specifies the time elapsed since the last frame update. Likely, it's going
     *                    to be somewhere around 50Hz. For the most part, you might not need this.
     */
    open fun physicsUpdate(engine: DropbearEngine, deltaTime: Float) {}

    /**
     * This function is called at the end of the script execution.
     *
     * It is run at the end of execution of a scene, such as when the scene switches. It is also ran once throughout
     * the lifecycle of a script class. It is best to think about it like `sceneExit()` instead of `appExit()`
     *
     * It would be used to clean up any memory related resources (such as `SceneLoadHandle` or any memory related items).
     *
     * # Note
     *
     * The script module does not lose state (such as variables) when destroyed. It is cached internally (within the system manager),
     * therefore counters and other related stuff will not lose track.
     */
    open fun destroy(engine: DropbearEngine) {}

    /**
     * Internal: This attaches the [DropbearEngine] fascade (typically created through some external location)
     * to the existing system to be used.
     */
    fun attachEngine(engine: DropbearEngine) {
        engineRef = engine
        currentEntity?.engine = engine
    }

    /**
     * Internal: Sets the current entity of this [System] to something.
     */
    fun setCurrentEntity(entity: Long) {
        val engine = engineRef ?: run {
            currentEntity = null
            return
        }

        val reference = EntityRef(EntityId(entity))
        reference.engine = engine
        currentEntity = reference
    }

    /**
     * Internal: Clears the current entity used.
     */
    fun clearCurrentEntity() {
        currentEntity = null
    }
}
