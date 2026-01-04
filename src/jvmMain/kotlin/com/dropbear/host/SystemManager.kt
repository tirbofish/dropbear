package com.dropbear.host

import com.dropbear.DropbearEngine
import com.dropbear.ecs.System
import com.dropbear.logging.LogLevel
import com.dropbear.logging.LogWriter
import com.dropbear.logging.Logger
import com.dropbear.logging.StdoutWriter

@Suppress("UNUSED")
class SystemManager(
    jarPath: String,
    private val engine: DropbearEngine,
    logWriter: LogWriter? = null,
    logLevel: LogLevel? = null,
    logTarget: String = "dropbear"
) {
    private val hotSwapUtility = HotSwapUtility(jarPath, "com.dropbear.decl.RunnableRegistry")
    private var registryInstance: Any? = null
    private var registryClass: Class<*>? = null
    private val activeSystems = mutableMapOf<String, MutableList<System>>()

    private fun instantiateSystemsForTag(tag: String): MutableList<System> {
        val existing = activeSystems[tag]
        if (existing != null) return existing

        val instantiateMethod = registryClass?.getMethod("instantiateScripts", String::class.java)
        val instances = instantiateMethod?.invoke(registryInstance, tag) as? List<*>

        val typedInstances = mutableListOf<System>()
        if (instances != null) {
            for (instance in instances) {
                val typed = instance as? System
                if (typed == null) {
                    Logger.warn(
                        "Skipping script instance that does not extend com.dropbear.System: ${instance?.javaClass?.name}"
                    )
                    continue
                }

                try {
                    typed.attachEngine(engine)
                    typed.clearCurrentEntity()
                    typedInstances.add(typed)
                    Logger.trace("Instantiated system: ${typed.javaClass.name} for tag: $tag")
                } catch (ex: Exception) {
                    Logger.error("Failed to instantiate system ${typed.javaClass.name}: ${ex.message}")
                }
            }
        } else {
            Logger.warn("No systems found for tag: $tag")
        }

        activeSystems[tag] = typedInstances
        return typedInstances
    }

    private fun destroySystems(tag: String, systems: List<System>) {
        if (systems.isEmpty()) return

        Logger.debug("Destroying ${systems.size} system(s) for tag: $tag")
        for (system in systems) {
            try {
                system.attachEngine(engine)
                system.clearCurrentEntity()
                system.destroy(engine)
            } catch (ex: Exception) {
                Logger.error("Failed to destroy system ${system.javaClass.name} for tag $tag: ${ex.message}")
            } finally {
                try {
                    system.clearCurrentEntity()
                } catch (_: Exception) {
                    // ignore
                }
            }
        }
    }

    init {
        val writerToUse = logWriter ?: StdoutWriter()
        Logger.init(writerToUse, logLevel ?: LogLevel.INFO, logTarget)
        Logger.info("SystemManager: Initialised with jarPath: $jarPath, " +
                "logWriter: $writerToUse, " +
                "logLevel: $logLevel, " +
                "logTarget: $logTarget")

        val (instance, clazz) = loadRegistry()
        registryInstance = instance
        registryClass = clazz
    }

    private fun loadRegistry(): Pair<Any, Class<*>> {
        Logger.debug("Loading RunnableRegistry instance...")
        val instance = hotSwapUtility.getInstance(emptyArray(), emptyArray())
        Logger.debug("RunnableRegistry instance loaded successfully.")
        return instance to instance.javaClass
    }

    fun loadSystemsForTag(tag: String) {
        Logger.debug("Loading systems for tag: $tag")

        if (activeSystems.containsKey(tag)) {
            Logger.trace("Systems already loaded for tag: $tag; re-running load() on existing instances")
            val systems = activeSystems[tag] ?: return
            for (system in systems) {
                try {
                    system.attachEngine(engine)
                    system.clearCurrentEntity()
                    system.load(engine)
                } catch (ex: Exception) {
                    Logger.error("Failed to load system ${system.javaClass.name} for tag $tag: ${ex.message}")
                } finally {
                    try {
                        system.clearCurrentEntity()
                    } catch (_: Exception) {
                        // ignore
                    }
                }
            }
            return
        }

        val systems = instantiateSystemsForTag(tag)
        for (system in systems) {
            try {
                system.attachEngine(engine)
                system.clearCurrentEntity()
                system.load(engine)
            } catch (ex: Exception) {
                Logger.error("Failed to load system ${system.javaClass.name} for tag $tag: ${ex.message}")
            } finally {
                try {
                    system.clearCurrentEntity()
                } catch (_: Exception) {
                    // ignore
                }
            }
        }

        Logger.debug("Loaded ${systems.size} systems for tag: $tag")
    }

    fun loadSystemsForEntities(tag: String, entityIds: LongArray) {
        Logger.trace("Loading systems for tag: $tag with ${entityIds.size} entities")

        if (entityIds.isEmpty()) {
            loadSystemsForTag(tag)
            return
        }

        val systems = instantiateSystemsForTag(tag)

        for (entityId in entityIds) {
            for (system in systems) {
                try {
                    system.attachEngine(engine)
                    system.setCurrentEntity(entityId)
                    system.load(engine)
                } catch (ex: Exception) {
                    Logger.error("Failed to load system ${system.javaClass.name} for entity $entityId: ${ex.message}")
                } finally {
                    try {
                        system.clearCurrentEntity()
                    } catch (_: Exception) {
                        // ignore
                    }
                }
            }
        }
    }

    fun updateAllSystems(deltaTime: Float) {
        Logger.trace("Updating all systems")
        for ((tag, systems) in activeSystems) {
            updateSystemsInternal(tag, systems, deltaTime)
        }
    }

    fun physicsUpdateAllSystems(deltaTime: Float) {
        Logger.trace("Physics updating all systems")
        for ((tag, systems) in activeSystems) {
            physicsUpdateSystemsInternal(tag, systems, deltaTime)
        }
    }

    fun updateSystemsByTag(tag: String, deltaTime: Float) {
        Logger.trace("Updating systems for tag: $tag")
        val systems = activeSystems[tag] ?: return
        updateSystemsInternal(tag, systems, deltaTime)
    }

    fun physicsUpdateSystemsByTag(tag: String, deltaTime: Float) {
        Logger.trace("Physics updating systems for tag: $tag")
        val systems = activeSystems[tag] ?: return
        physicsUpdateSystemsInternal(tag, systems, deltaTime)
    }

    fun updateSystemsForEntities(tag: String, entityIds: LongArray, deltaTime: Float) {
        Logger.trace("Updating systems for tag: $tag with ${entityIds.size} entities")
        val systems = activeSystems[tag] ?: return

        if (systems.isEmpty()) {
            return
        }

        if (entityIds.isEmpty()) {
            updateSystemsInternal(tag, systems, deltaTime)
            return
        }

        for (entityId in entityIds) {
            for (system in systems) {
                try {
                    system.attachEngine(engine)
                    system.setCurrentEntity(entityId)
                    system.update(engine, deltaTime)
                } catch (ex: Exception) {
                    Logger.error("Failed to update system ${system.javaClass.name} for entity $entityId: ${ex.message}")
                }
            }
        }

        for (system in systems) {
            system.clearCurrentEntity()
        }
    }

    fun physicsUpdateSystemsForEntities(tag: String, entityIds: LongArray, deltaTime: Float) {
        Logger.trace("Physics updating systems for tag: $tag with ${entityIds.size} entities")
        val systems = activeSystems[tag] ?: return

        if (systems.isEmpty()) {
            return
        }

        if (entityIds.isEmpty()) {
            physicsUpdateSystemsInternal(tag, systems, deltaTime)
            return
        }

        for (entityId in entityIds) {
            for (system in systems) {
                try {
                    system.attachEngine(engine)
                    system.setCurrentEntity(entityId)
                    system.physicsUpdate(engine, deltaTime)
                } catch (ex: Exception) {
                    Logger.error("Failed to physics update system ${system.javaClass.name} for entity $entityId: ${ex.message}")
                }
            }
        }

        for (system in systems) {
            system.clearCurrentEntity()
        }
    }

    fun collisionEvent(tag: String, entityId: Long, event: com.dropbear.physics.CollisionEvent) {
        val systems = activeSystems[tag] ?: return

        for (system in systems) {
            try {
                system.attachEngine(engine)
                system.setCurrentEntity(entityId)
                system.collisionEvent(engine, event)
            } catch (ex: Exception) {
                Logger.error("Failed to deliver collision event to ${system.javaClass.name} for entity $entityId: ${ex.message}")
            }
        }

        for (system in systems) {
            system.clearCurrentEntity()
        }
    }

    fun collisionForceEvent(tag: String, entityId: Long, event: com.dropbear.physics.ContactForceEvent) {
        val systems = activeSystems[tag] ?: return

        for (system in systems) {
            try {
                system.attachEngine(engine)
                system.setCurrentEntity(entityId)
                system.collisionForceEvent(engine, event)
            } catch (ex: Exception) {
                Logger.error("Failed to deliver contact-force event to ${system.javaClass.name} for entity $entityId: ${ex.message}")
            }
        }

        for (system in systems) {
            system.clearCurrentEntity()
        }
    }

    private fun updateSystemsInternal(tag: String, systems: List<System>, deltaTime: Float) {
        for (system in systems) {
            try {
                system.attachEngine(engine)
                system.clearCurrentEntity()
                system.update(engine, deltaTime)
            } catch (ex: Exception) {
                Logger.error("Failed to update system ${system.javaClass.name} for tag $tag: ${ex.message}")
            }
        }
    }

    private fun physicsUpdateSystemsInternal(tag: String, systems: List<System>, deltaTime: Float) {
        for (system in systems) {
            try {
                system.attachEngine(engine)
                system.clearCurrentEntity()
                system.physicsUpdate(engine, deltaTime)
            } catch (ex: Exception) {
                Logger.error("Failed to physics update system ${system.javaClass.name} for tag $tag: ${ex.message}")
            }
        }
    }

    fun reloadJar(newJarPath: String) {
        Logger.info("Reloading systems with new jar path: $newJarPath")

        unloadAllSystems()

        hotSwapUtility.reloadJar(newJarPath)

        val (instance, clazz) = loadRegistry()
        registryInstance = instance
        registryClass = clazz

        val reloadMethod = registryClass?.getMethod("reload")
        reloadMethod?.invoke(registryInstance)
        Logger.info("JAR loaded successfully.")
    }

    fun unloadSystemsByTag(tag: String) {
        val systems = activeSystems.remove(tag)
        if (systems != null) {
            destroySystems(tag, systems)
        }
    }

    /**
     * Runs `System.destroy()` for the given tag without unloading/removing the instances.
     *
     * This is intended for scene switches: scripts leave scope and should clean up resources,
     * but the classes/instances remain cached until the application stops.
     */
    fun destroySystemsByTag(tag: String) {
        val systems = activeSystems[tag] ?: return
        destroySystems(tag, systems)
    }

    fun unloadAllSystems() {
        val snapshot = activeSystems.toMap()
        activeSystems.clear()

        for ((tag, systems) in snapshot) {
            destroySystems(tag, systems)
        }
    }

    fun getSystemCount(tag: String): Int = activeSystems[tag]?.size ?: 0

    fun getTotalSystemCount(): Int = activeSystems.values.sumOf { it.size }

    fun getActiveTags(): Set<String> = activeSystems.keys.toSet()

    fun hasSystemsForTag(tag: String): Boolean = activeSystems[tag]?.isNotEmpty() == true
}


