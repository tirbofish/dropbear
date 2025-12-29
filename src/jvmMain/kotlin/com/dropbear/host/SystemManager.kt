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

        val instantiateMethod = registryClass?.getMethod("instantiateScripts", String::class.java)
        val systems = instantiateMethod?.invoke(registryInstance, tag) as? List<*>

        val loadedSystems = mutableListOf<System>()

        if (systems != null) {
            for (system in systems) {
                val typed = system as? System
                if (typed == null) {
                    Logger.warn("Skipping script instance that does not extend com.dropbear.System: ${system?.javaClass?.name}")
                    continue
                }

                try {
                    typed.attachEngine(engine)
                    typed.clearCurrentEntity()
                    typed.load(engine)
                    loadedSystems.add(typed)
                    Logger.trace("Loaded system: ${typed.javaClass.name} for tag: $tag")
                } catch (ex: Exception) {
                    Logger.error("Failed to load system ${typed.javaClass.name}: ${ex.message}")
                }
            }
        } else {
            Logger.warn("No systems found for tag: $tag")
        }

        activeSystems[tag] = loadedSystems
        Logger.debug("Loaded ${loadedSystems.size} systems for tag: $tag")
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


