package com.dropbear.ffi

import com.dropbear.Camera
import com.dropbear.EntityId
import com.dropbear.EntityRef
import com.dropbear.EntityTransform
import com.dropbear.asset.TextureHandle
import com.dropbear.exception.DropbearNativeException
import com.dropbear.exceptionOnError
import com.dropbear.ffi.JNINative.*
import com.dropbear.ffi.InputStateNative.*
import com.dropbear.ffi.DropbearEngineNative.*
import com.dropbear.ffi.components.HierarchyNative.*
import com.dropbear.ffi.components.CameraNative.*
import com.dropbear.ffi.components.ColliderNative
import com.dropbear.ffi.components.LabelNative.*
import com.dropbear.ffi.components.MeshRendererNative.*
import com.dropbear.ffi.components.EntityTransformNative.*
import com.dropbear.ffi.components.CustomPropertiesNative.*
import com.dropbear.ffi.components.RigidBodyNative
import com.dropbear.input.Gamepad
import com.dropbear.input.GamepadButton
import com.dropbear.input.KeyCode
import com.dropbear.input.MouseButton
import com.dropbear.input.MouseButtonCodes
import com.dropbear.logging.Logger
import com.dropbear.math.Transform
import com.dropbear.math.Vector2D
import com.dropbear.physics.Collider
import com.dropbear.physics.Index
import com.dropbear.physics.RigidBody
import com.dropbear.scene.SceneLoadHandle
import com.dropbear.scene.SceneLoadStatus
import com.dropbear.utils.Progress

actual class NativeEngine {
    private var worldHandle: Long = 0L
    private var inputHandle: Long = 0L
    private var commandBufferHandle: Long = 0L
    private var assetHandle: Long = 0L
    private var sceneLoaderHandle: Long = 0L
    private var physicsEngineHandle: Long = 0L

    @JvmName("init")
    fun init(ctx: DropbearContext) {
        this.worldHandle = ctx.worldHandle
        this.inputHandle = ctx.inputHandle
        this.commandBufferHandle = ctx.commandBufferHandle
        this.assetHandle = ctx.assetHandle
        this.sceneLoaderHandle = ctx.sceneLoaderHandle
        this.physicsEngineHandle = ctx.physicsEngineHandle

        if (this.worldHandle < 0L) {
            Logger.error("NativeEngine: Error - Invalid world handle received!")
            return
        }
        if (this.inputHandle < 0L) {
            Logger.error("NativeEngine: Error - Invalid input handle received!")
            return
        }
        if (this.commandBufferHandle < 0L) {
            Logger.error("NativeEngine: Error - Invalid graphics handle received!")
            return
        }
        if (this.assetHandle < 0L) {
            Logger.error("NativeEngine: Error - Invalid asset handle received!")
            return
        }
        if (this.sceneLoaderHandle < 0L) {
            Logger.error("NativeEngine: Error - Invalid scene loader handle received!")
            return
        }
        if (this.physicsEngineHandle < 0L) {
            Logger.error("NativeEngine: Error - Invalid physics handle received!")
            return
        }
    }

    actual fun getEntityLabel(entityHandle: Long) : String? {
        val result = getEntityLabel(worldHandle, entityHandle) ?: if (exceptionOnError) {
            throw DropbearNativeException("Unable to get entity label for entity $entityHandle")
        } else {
            return null
        }
        return result
    }

    actual fun getEntity(label: String): Long? {
        val result = getEntity(worldHandle, label)
        return if (result == -1L) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get entity: returned -1")
            } else {
                null
            }
        } else if (result == 0L) {
            null
        } else {
            result
        }
    }


    actual fun getTransform(entityId: EntityId): EntityTransform? {
        return getTransform(worldHandle, entityId.id)
    }

    actual fun propagateTransform(entityId: EntityId): Transform? {
        return propagateTransform(worldHandle, entityId.id)
    }

    actual fun setTransform(entityId: EntityId, transform: EntityTransform) {
        return setTransform(worldHandle, entityId.id, transform)
    }

    actual fun printInputState() {
        return printInputState(inputHandle)
    }

    actual fun isKeyPressed(key: KeyCode): Boolean {
        return isKeyPressed(inputHandle, key.ordinal)
    }

    actual fun getMousePosition(): Vector2D? {
        val result = getMousePosition(inputHandle);
        return Vector2D(result[0].toDouble(), result[1].toDouble())
    }

    actual fun isMouseButtonPressed(button: MouseButton): Boolean {
        val buttonCode: Int = when (button) {
            MouseButton.Left -> MouseButtonCodes.LEFT
            MouseButton.Right -> MouseButtonCodes.RIGHT
            MouseButton.Middle -> MouseButtonCodes.MIDDLE
            MouseButton.Back -> MouseButtonCodes.BACK
            MouseButton.Forward -> MouseButtonCodes.FORWARD
            is MouseButton.Other -> button.value
        }

        return isMouseButtonPressed(inputHandle, buttonCode)
    }

    actual fun getConnectedGamepads(): List<Gamepad> {
        val result = getConnectedGamepads(inputHandle)
        return result.toList()
    }

    actual fun isGamepadButtonPressed(id: Long, button: GamepadButton): Boolean {
        return isGamepadButtonPressed(inputHandle, id, button.ordinal)
    }

    actual fun getMouseDelta(): Vector2D? {
        val result = getMouseDelta(inputHandle);
        return Vector2D(result[0].toDouble(), result[1].toDouble())
    }

    actual fun isCursorLocked(): Boolean {
        return isCursorLocked(inputHandle)
    }

    actual fun setCursorLocked(locked: Boolean) {
        setCursorLocked(inputHandle, commandBufferHandle, locked)
    }

    actual fun getLastMousePos(): Vector2D? {
        val result = getLastMousePos(inputHandle);
        return Vector2D(result[0].toDouble(), result[1].toDouble())
    }

    actual fun getStringProperty(entityHandle: Long, label: String): String? {
        return getStringProperty(worldHandle, entityHandle, label)
    }

    actual fun getIntProperty(entityHandle: Long, label: String): Int? {
        val result = getIntProperty(worldHandle, entityHandle, label)
        return if (result == 650911) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get integer property for entity $label")
            } else {
                null
            }
        } else {
            result
        }
    }

    actual fun getLongProperty(entityHandle: Long, label: String): Long? {
        val result = getLongProperty(worldHandle, entityHandle, label)
        return if (result == 6509112938) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get long property for entity $label")
            } else {
                null
            }
        } else {
            result
        }
    }

    actual fun getFloatProperty(entityHandle: Long, label: String): Float? {
        val result = getFloatProperty(worldHandle, entityHandle, label)
        return if (result.isNaN()) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get float property for entity $label")
            } else {
                null
            }
        } else {
            result.toFloat()
        }
    }

    actual fun getDoubleProperty(entityHandle: Long, label: String): Double? {
        val result = getFloatProperty(worldHandle, entityHandle, label)
        return if (result.isNaN()) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get double (float) property")
            } else {
                null
            }
        } else {
            result
        }
    }

    actual fun getBoolProperty(entityHandle: Long, label: String): Boolean? {
        return getBoolProperty(worldHandle, entityHandle, label)
    }

    actual fun getVec3Property(entityHandle: Long, label: String): FloatArray? {
        return getVec3Property(worldHandle, entityHandle, label)
    }

    actual fun setStringProperty(entityHandle: Long, label: String, value: String) {
        setStringProperty(worldHandle, entityHandle, label, value)
    }

    actual fun setIntProperty(entityHandle: Long, label: String, value: Int) {
        setIntProperty(worldHandle, entityHandle, label, value)
    }

    actual fun setLongProperty(entityHandle: Long, label: String, value: Long) {
        setLongProperty(worldHandle, entityHandle, label, value)
    }

    actual fun setFloatProperty(entityHandle: Long, label: String, value: Double) {
        setFloatProperty(worldHandle, entityHandle, label, value)
    }

    actual fun setBoolProperty(entityHandle: Long, label: String, value: Boolean) {
        setBoolProperty(worldHandle, entityHandle, label, value)
    }

    actual fun setVec3Property(entityHandle: Long, label: String, value: FloatArray) {
        setVec3Property(worldHandle, entityHandle, label, value)
    }

    actual fun getCamera(label: String): Camera? {
        return getCamera(worldHandle, label)
    }

    actual fun getAttachedCamera(entityId: EntityId): Camera? {
        return getAttachedCamera(worldHandle, entityId.id)
    }

    actual fun setCamera(camera: Camera) {
        setCamera(worldHandle, camera)
    }

    actual fun isCursorHidden(): Boolean {
        return isCursorHidden(inputHandle)
    }

    actual fun setCursorHidden(hidden: Boolean) {
        setCursorHidden(inputHandle, commandBufferHandle, hidden)
    }

    actual fun getModel(entityHandle: Long): Long? {
        val result = getModel(worldHandle, entityHandle)
        return if (result == -1L) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get model for entity $entityHandle")
            } else {
                null
            }
        } else if (result == 0L) {
            null
        } else {
            result
        }
    }

    actual fun setModel(entityHandle: Long, modelHandle: Long) {
        setModel(worldHandle, assetHandle, entityHandle, modelHandle)
    }

    actual fun getTexture(entityHandle: Long, name: String): Long? {
        val result = getTexture(worldHandle, assetHandle, entityHandle, name)
        return if (result == -1L) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get texture for entity $entityHandle")
            } else {
                null
            }
        } else if (result == 0L) {
            null
        } else {
            result
        }
    }

    actual fun setTextureOverride(entityHandle: Long, oldMaterialName: String, newTextureHandle: TextureHandle) {
        return setTexture(
            worldHandle,
            assetHandle,
            entityHandle,
            oldMaterialName,
            newTextureHandle.raw()
        )
    }

    actual fun getTextureName(textureHandle: Long): String? {
        return getTextureName(assetHandle, textureHandle)
    }

    actual fun isUsingModel(entityHandle: Long, modelHandle: Long): Boolean {
        return isUsingModel(worldHandle, entityHandle, modelHandle)
    }

    actual fun isUsingTexture(entityHandle: Long, textureHandle: Long): Boolean {
        return isUsingTexture(worldHandle, entityHandle, textureHandle)
    }

    actual fun getAsset(eucaURI: String): Long? {
        val result = getAsset(assetHandle, eucaURI)
        return if (result == -1L) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get asset for URI $eucaURI")
            } else {
                null
            }
        } else if (result == 0L) {
            // no asset found
            null
        } else {
            result
        }
    }

    actual fun isModelHandle(id: Long): Boolean {
        return isModelHandle(assetHandle, id)
    }

    actual fun isTextureHandle(id: Long): Boolean {
        return isTextureHandle(assetHandle, id)
    }

    actual fun getAllTextures(entityHandle: Long): Array<String> {
        return getAllTextures(worldHandle, entityHandle) ?: emptyArray()
    }

    actual fun getChildren(entityId: EntityId): Array<EntityRef>? {
        val result = getChildren(worldHandle, entityId.id)
        // i shouldn't expect it to return null unless an error, otherwise it must
        // return an empty array
        if (result == null) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to query for all children for entity ${entityId.id}")
            } else {
                return null
            }
        } else {
            val entityRefs = mutableListOf<EntityRef>()
            result.forEach { e ->
                entityRefs.add(EntityRef(EntityId(e)))
            }
            return entityRefs.toTypedArray() // must be an array so it cannot be mutated
        }
    }

    actual fun getChildByLabel(entityId: EntityId, label: String): EntityRef? {
        val result = getChildByLabel(worldHandle, entityId.id, label)
        return if (result == -1L) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get child by label $entityId $label")
            } else {
                null
            }
        } else if (result == -2L) {
            null
        } else {
            EntityRef(EntityId(result))
        }
    }

    actual fun getParent(entityId: EntityId): EntityRef? {
        val result = getParent(worldHandle, entityId.id)
        return if (result == -1L) {
            if (exceptionOnError) {
                throw DropbearNativeException("Unable to get parent of entity $entityId")
            } else {
                null
            }
        } else if (result == -2L) {
            null
        } else {
            EntityRef(EntityId(result))
        }
    }

    actual fun quit() {
        quit(commandBufferHandle)
    }

    actual fun switchToSceneImmediate(sceneName: String) {
        SceneNative.switchToSceneImmediate(commandBufferHandle, sceneName)
    }

    actual fun loadSceneAsync(sceneName: String): SceneLoadHandle? {
        return SceneNative.loadSceneAsync(commandBufferHandle, sceneLoaderHandle, sceneName, this)
    }

    actual fun loadSceneAsync(sceneName: String, loadingScene: String): SceneLoadHandle? {
        return SceneNative.loadSceneAsync(commandBufferHandle, sceneLoaderHandle, sceneName, loadingScene, this)
    }

    actual fun switchToSceneAsync(sceneLoadHandle: SceneLoadHandle) {
        SceneNative.switchToSceneAsync(commandBufferHandle, sceneLoadHandle) // will throw exception from JNI interface
    }

    actual fun getSceneLoadProgress(sceneLoadHandle: SceneLoadHandle): Progress {
        return SceneNative.getSceneLoadProgress(sceneLoaderHandle, sceneLoadHandle)
    }

    actual fun getSceneLoadStatus(sceneLoadHandle: SceneLoadHandle): SceneLoadStatus {
        return SceneNative.getSceneLoadStatus(sceneLoaderHandle, sceneLoadHandle)
    }

    actual fun setPhysicsEnabled(entityId: Long, enabled: Boolean) {
        return PhysicsNative.setPhysicsEnabled(worldHandle, physicsEngineHandle, entityId, enabled)
    }

    actual fun isPhysicsEnabled(entityId: Long): Boolean {
        return PhysicsNative.isPhysicsEnabled(worldHandle, physicsEngineHandle, entityId)
    }

    actual fun getRigidBody(entityId: Long): RigidBody? {
        return PhysicsNative.getRigidBody(worldHandle, physicsEngineHandle, entityId)
    }

    actual fun getAllColliders(entityId: Long): List<Collider> {
        val result = PhysicsNative.getAllColliders(worldHandle, physicsEngineHandle, entityId)
        return result.toList()
    }

    actual fun applyImpulse(index: Index, x: Double, y: Double, z: Double) {
        return RigidBodyNative.applyImpulse(physicsEngineHandle, index, x, y, z)
    }

    actual fun applyTorqueImpulse(index: Index, x: Double, y: Double, z: Double) {
        return RigidBodyNative.applyTorqueImpulse(physicsEngineHandle, index, x, y, z)
    }

    actual fun setRigidbody(rigidBody: RigidBody) {
        return RigidBodyNative.setRigidBody(worldHandle, physicsEngineHandle, rigidBody)
    }

    actual fun getChildColliders(index: Index): List<Collider> {
        return RigidBodyNative.getChildColliders(worldHandle, physicsEngineHandle, index).toList()
    }

    actual fun setCollider(collider: Collider) {
        return ColliderNative.setCollider(worldHandle, physicsEngineHandle, collider)
    }
}