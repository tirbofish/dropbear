package com.dropbear.host

import com.dropbear.logging.Logger
import java.lang.reflect.InvocationTargetException
import java.net.URLClassLoader
import java.nio.file.Path

class HotSwapUtility(
    private var jarFilePath: String,
    private var className: String
) {
    private lateinit var classLoader: URLClassLoader

    init {
        initialiseClassLoader()
    }

    private fun initialiseClassLoader() {
        try {
            val urls = arrayOf(Path.of(jarFilePath).toUri().toURL())
            classLoader = URLClassLoader(urls, HotSwapUtility::class.java.classLoader)
        } catch (e: Exception) {
            Logger.error("Failed to initialise class loader: ${e.message}")
            e.printStackTrace()
        }
    }

    @Throws(
        ClassNotFoundException::class,
        NoSuchMethodException::class,
        IllegalAccessException::class,
        InvocationTargetException::class,
        InstantiationException::class
    )
    fun getInstance(parameterTypes: Array<Class<*>>, args: Array<out Any?>): Any {
        try {
            val clazz = classLoader.loadClass(className)
            if (clazz.isAnnotationPresent(Metadata::class.java)) {
                try {
                    val instanceField = clazz.getDeclaredField("INSTANCE")
                    return instanceField.get(null)
                } catch (e: NoSuchFieldException) {
                    Logger.error("Failed to get instance of class: ${e.message}")
                }
            }
            val constructor = clazz.getConstructor(*parameterTypes)
            return constructor.newInstance(*args)
        } catch (e: ClassNotFoundException) {
            throw ClassNotFoundException("Class not found: $className. It seems like you have not generated your magna-carta manifest file.\n$e")
        } catch (e: Exception) {
            throw e
        }
    }

    fun reloadJar(newJarFilePath: String) {
        try {
            classLoader.close()
        } catch (e: Exception) {
            e.printStackTrace()
        }
        jarFilePath = newJarFilePath
        initialiseClassLoader()
    }
}