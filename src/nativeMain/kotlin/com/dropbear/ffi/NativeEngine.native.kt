@file:OptIn(ExperimentalForeignApi::class, ExperimentalNativeApi::class)
@file:Suppress("EXPECT_ACTUAL_CLASSIFIERS_ARE_IN_BETA_WARNING")

/// guys how do i remove the reinterpret error its genuinely pmo and intellij keeps
/// on catching it.

package com.dropbear.ffi

import com.dropbear.Camera
import com.dropbear.EntityId
import com.dropbear.EntityRef
import com.dropbear.EntityTransform
import com.dropbear.asset.TextureHandle
import com.dropbear.exception.DropbearNativeException
import com.dropbear.exception.PrematureSceneSwitchException
import com.dropbear.exceptionOnError
import com.dropbear.ffi.generated.*
import com.dropbear.input.Gamepad
import com.dropbear.input.GamepadButton
import com.dropbear.input.KeyCode
import com.dropbear.input.MouseButton
import com.dropbear.input.MouseButtonCodes
import com.dropbear.logging.Logger
import com.dropbear.math.Transform
import com.dropbear.math.Vector2D
import com.dropbear.physics.AxisLock
import com.dropbear.physics.Collider
import com.dropbear.physics.Index
import com.dropbear.physics.RigidBody
import com.dropbear.scene.SceneLoadHandle
import com.dropbear.scene.SceneLoadStatus
import com.dropbear.utils.Progress
import kotlinx.cinterop.*
import kotlin.experimental.ExperimentalNativeApi

actual class NativeEngine {
    private var worldHandle: COpaquePointer? = null
    private var inputHandle: COpaquePointer? = null
    private var commandBufferHandle: COpaquePointer? = null
    private var assetHandle: COpaquePointer? = null
    private var sceneLoaderHandle: COpaquePointer? = null
    private var physicsEngineHandle: COpaquePointer? = null

    @Suppress("unused")
    fun init(
        ctx: DropbearContext?
    ) {
        this.worldHandle = ctx?.world?.rawValue?.let { interpretCPointer(it) }
        this.inputHandle = ctx?.input?.rawValue?.let { interpretCPointer(it) }
        this.commandBufferHandle = ctx?.graphics?.rawValue?.let { interpretCPointer(it) }
        this.assetHandle = ctx?.assets?.rawValue?.let { interpretCPointer(it) }
        this.sceneLoaderHandle = ctx?.scene_loader?.rawValue?.let { interpretCPointer(it) }
        this.physicsEngineHandle = ctx?.physics_engine?.rawValue?.let { interpretCPointer(it) }

        // if release, always enable exceptionOnError
        if (!Platform.isDebugBinary) {
            exceptionOnError = true
        }

        if (this.worldHandle == null) {
            Logger.error("NativeEngine: Error - Invalid world handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid world handle received!")
            }
        }
        if (this.inputHandle == null) {
            Logger.error("NativeEngine: Error - Invalid input handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid input handle received!")
            }
        }
        if (this.commandBufferHandle == null) {
            Logger.error("NativeEngine: Error - Invalid graphics handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid graphics handle received!")
            }
        }
        if (this.assetHandle == null) {
            Logger.error("NativeEngine: Error - Invalid asset handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid asset handle received!")
            }
        }
        if (this.physicsEngineHandle == null) {
            Logger.error("NativeEngine: Error - Invalid physics engine handle received!")
            if (exceptionOnError) {
                throw DropbearNativeException("init failed - Invalid physics engine handle received!")
            }
        }
    }

    actual fun getEntity(label: String): Long? {
        val world = worldHandle ?: return null
        memScoped {
            val outEntity = alloc<LongVar>()
            val result = dropbear_get_entity(
                label = label,
                world_ptr = world.reinterpret(),
                out_entity = outEntity.ptr
            )
            return if (result == 0) outEntity.value else if (exceptionOnError) {
                throw DropbearNativeException("getEntity failed with code: $result")
            } else {
                println("getEntity failed with code: $result")
                null
            }
        }
    }

    actual fun getEntityLabel(entityHandle: Long): String? {
        val world = worldHandle ?: return null
        memScoped {
            val bufferSize = 256
            val outLabel = allocArray<ByteVar>(bufferSize)

            val result = dropbear_get_entity_name(
                world_ptr = world.reinterpret(),
                entity_id = entityHandle,
                out_name = outLabel,
                max_len = bufferSize.toULong()
            )

            if (result == 0) {
                return outLabel.toKString()
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getEntityLabel failed with code for entity '$entityHandle': $result")
                } else {
                    println("getEntityLabel failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getTransform(entityId: EntityId): EntityTransform? {
        val world = worldHandle ?: return null
        memScoped {
            val outTransform = alloc<NativeEntityTransform>()
            val result = dropbear_get_transform(
                world_ptr = world.reinterpret(),
                entity_handle = entityId.id,
                out_transform = outTransform.ptr
            )
            if (result == 0) {
                return EntityTransform(
                    local = Transform(
                        position = com.dropbear.math.Vector3D(
                            outTransform.local.position_x,
                            outTransform.local.position_y,
                            outTransform.local.position_z
                        ),
                        rotation = com.dropbear.math.QuaternionD(
                            outTransform.local.rotation_x,
                            outTransform.local.rotation_y,
                            outTransform.local.rotation_z,
                            outTransform.local.rotation_w
                        ),
                        scale = com.dropbear.math.Vector3D(
                            outTransform.local.scale_x, outTransform.local.scale_y,
                            outTransform.local.scale_z
                        )
                    ),
                    world = Transform(
                        position = com.dropbear.math.Vector3D(
                            outTransform.world.position_x,
                            outTransform.world.position_y,
                            outTransform.world.position_z
                        ),
                        rotation = com.dropbear.math.QuaternionD(
                            outTransform.world.rotation_x,
                            outTransform.world.rotation_y,
                            outTransform.world.rotation_z,
                            outTransform.world.rotation_w
                        ),
                        scale = com.dropbear.math.Vector3D(
                            outTransform.world.scale_x,
                            outTransform.world.scale_y,
                            outTransform.world.scale_z
                        )
                    )
                )
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getTransform failed with code: $result")
                } else {
                    println("getTransform failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun propagateTransform(entityId: EntityId): Transform? {
        val world = worldHandle ?: return null
        memScoped {
            val outTransform = alloc<NativeTransform>()
            val result = dropbear_propagate_transform(
                world_ptr = world.reinterpret(),
                entity_id = entityId.id,
                out_transform = outTransform.ptr
            )
            if (result == 0) {
                return Transform(
                    position = com.dropbear.math.Vector3D(
                        outTransform.position_x,
                        outTransform.position_y,
                        outTransform.position_z
                    ),
                    rotation = com.dropbear.math.QuaternionD(
                        outTransform.rotation_x,
                        outTransform.rotation_y,
                        outTransform.rotation_z,
                        outTransform.rotation_w
                    ),
                    scale = com.dropbear.math.Vector3D(
                        outTransform.scale_x,
                        outTransform.scale_y,
                        outTransform.scale_z
                    )
                )
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("propagateTransform failed with code: $result")
                } else {
                    println("propagateTransform failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun setTransform(entityId: EntityId, transform: EntityTransform) {
        val worldHandle = worldHandle ?: return
        memScoped {
            val nativeTransform = cValue<NativeEntityTransform> {
                local.position_x = transform.local.position.x
                local.position_y = transform.local.position.y
                local.position_z = transform.local.position.z
                local.rotation_w = transform.local.rotation.w
                local.rotation_x = transform.local.rotation.x
                local.rotation_y = transform.local.rotation.y
                local.rotation_z = transform.local.rotation.z
                local.scale_x = transform.local.scale.x
                local.scale_y = transform.local.scale.y
                local.scale_z = transform.local.scale.z

                world.position_x = transform.world.position.x
                world.position_y = transform.world.position.y
                world.position_z = transform.world.position.z
                world.rotation_w = transform.world.rotation.w
                world.rotation_x = transform.world.rotation.x
                world.rotation_y = transform.world.rotation.y
                world.rotation_z = transform.world.rotation.z
                world.scale_x = transform.world.scale.x
                world.scale_y = transform.world.scale.y
                world.scale_z = transform.world.scale.z
            }

            val result = dropbear_set_transform(
                world_ptr = worldHandle.reinterpret(),
                entity_id = entityId.id,
                transform = nativeTransform
            )

            if (result != 0) {
                if (exceptionOnError) {
                    throw DropbearNativeException("setTransform failed with code: $result")
                } else {
                    println("setTransform failed with code: $result")
                }
            }
        }
    }

    actual fun printInputState() {
        val input = inputHandle ?: return
        dropbear_print_input_state(input_ptr = input.reinterpret())
    }

    actual fun isKeyPressed(key: KeyCode): Boolean {
        val input = inputHandle ?: return false
        memScoped {
            val out = alloc<IntVar>()
            val result = dropbear_is_key_pressed(
                input.reinterpret(),
                key.ordinal,
                out.ptr
            )
            return if (result == 0) out.value != 0 else if (exceptionOnError) {
                throw DropbearNativeException("isKeyPressed failed with code: $result")
            } else {
                println("isKeyPressed failed with code: $result")
                false
            }
        }
    }

    actual fun getMousePosition(): Vector2D? {
        val input = inputHandle ?: return null
        memScoped {
            val xVar = alloc<FloatVar>()
            val yVar = alloc<FloatVar>()

            val result = dropbear_get_mouse_position(
                input.reinterpret(),
                xVar.ptr,
                yVar.ptr
            )

            if (result == 0) {
                val x = xVar.value.toDouble()
                val y = yVar.value.toDouble()
                return Vector2D(x, y)
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getMousePosition failed with code: $result")
                } else {
                    println("getMousePosition failed with code: $result")
                    return null
                }
            }
        }
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

        val input = inputHandle ?: return false

        memScoped {
            val pressedVar = alloc<IntVar>()

            val result = dropbear_is_mouse_button_pressed(
                input.reinterpret(),
                buttonCode,
                pressedVar.ptr
            )

            if (result == 0) {
                return pressedVar.value != 0
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("isMouseButtonPressed failed with code: $result")
                } else {
                    println("isMouseButtonPressed failed with code: $result")
                    return false
                }
            }
        }
    }

    actual fun getMouseDelta(): Vector2D? {
        val input = inputHandle ?: return null
        memScoped {
            val deltaXVar = alloc<FloatVar>()
            val deltaYVar = alloc<FloatVar>()

            val result = dropbear_get_mouse_delta(
                input.reinterpret(),
                deltaXVar.ptr,
                deltaYVar.ptr
            )

            if (result == 0) {
                val deltaX = deltaXVar.value.toDouble()
                val deltaY = deltaYVar.value.toDouble()
                return Vector2D(deltaX, deltaY)
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getMouseDelta failed with code: $result")
                } else {
                    println("getMouseDelta failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun isCursorLocked(): Boolean {
        val input = inputHandle ?: return false
        memScoped {
            val lockedVar = alloc<IntVar>()

            val result = dropbear_is_cursor_locked(
                input.reinterpret(),
                lockedVar.ptr
            )

            if (result == 0) {
                return lockedVar.value != 0
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("isCursorLocked failed with code: $result")
                } else {
                    println("isCursorLocked failed with code: $result")
                    return false
                }
            }
        }
    }

    actual fun setCursorLocked(locked: Boolean) {
        val lockedInt = if (locked) 1 else 0
        val input = inputHandle ?: return
        val graphics = commandBufferHandle ?: return

        val result = dropbear_set_cursor_locked(
            input.reinterpret(),
            graphics.reinterpret(),
            lockedInt
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setCursorLocked failed with code: $result")
            } else {
                println("setCursorLocked failed with code: $result")
            }
        }
    }

    actual fun getLastMousePos(): Vector2D? {
        val input = inputHandle ?: return null
        memScoped {
            val xVar = alloc<FloatVar>()
            val yVar = alloc<FloatVar>()

            val result = dropbear_get_last_mouse_pos(
                input.reinterpret(),
                xVar.ptr,
                yVar.ptr
            )

            if (result == 0) {
                val x = xVar.value.toDouble()
                val y = yVar.value.toDouble()
                return Vector2D(x, y)
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getLastMousePos failed with code: $result")
                } else {
                    println("getLastMousePos failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun isCursorHidden(): Boolean {
        val input = inputHandle ?: return false
        memScoped {
            val hiddenVar = alloc<IntVar>()

            val result = dropbear_is_cursor_hidden(
                input.reinterpret(),
                hiddenVar.ptr
            )

            if (result == 0) {
                return hiddenVar.value != 0
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("isCursorHidden failed with code: $result")
                } else {
                    println("isCursorHidden failed with code: $result")
                    return false
                }
            }
        }
    }

    actual fun setCursorHidden(hidden: Boolean) {
        val hiddenInt = if (hidden) 1 else 0
        val input = inputHandle ?: return
        val graphics = commandBufferHandle ?: return

        val result = dropbear_set_cursor_hidden(
            input.reinterpret(),
            graphics.reinterpret(),
            hiddenInt
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setCursorHidden failed with code: $result")
            } else {
                println("setCursorHidden failed with code: $result")
            }
        }
    }

    actual fun getConnectedGamepads(): List<Gamepad> {
        val input = inputHandle ?: return emptyList()
        memScoped {
            val gamepadsPtr = alloc<CPointerVar<com.dropbear.ffi.generated.Gamepad>>()
            val count = alloc<IntVar>()

            val result = dropbear_get_connected_gamepads(
                input.reinterpret(),
                gamepadsPtr.ptr,
                count.ptr
            )

            if (result == 0) {
                val gamepadArray = gamepadsPtr.value
                val gamepadCount = count.value

                if (gamepadArray == null || gamepadCount == 0) {
                    return emptyList()
                }

                return List(gamepadCount) { index ->
                    val nativeGamepad = gamepadArray[index]
                    Gamepad(
                        id = nativeGamepad.id.toLong(),
                        leftStickPosition = Vector2D(
                            x = nativeGamepad.left_stick_pos.x,
                            y = nativeGamepad.left_stick_pos.y
                        ),
                        rightStickPosition = Vector2D(
                            x = nativeGamepad.right_stick_pos.x,
                            y = nativeGamepad.right_stick_pos.y
                        ),
                        native = this@NativeEngine
                    )
                }
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getConnectedGamepads failed with code: $result")
                } else {
                    println("getConnectedGamepads failed with code: $result")
                    return emptyList()
                }
            }
        }
    }

    actual fun isGamepadButtonPressed(id: Long, button: GamepadButton): Boolean {
        val input = inputHandle ?: return false
        memScoped {
            val outPressed = alloc<IntVar>()

            val result = dropbear_is_gamepad_button_pressed(
                input_ptr = input.reinterpret(),
                gamepad_id = id,
                ordinal = button.ordinal,
                out_pressed = outPressed.ptr
            )

            return if (result == 0) {
                outPressed.value != 0
            } else if (exceptionOnError) {
                throw DropbearNativeException(
                    "isGamepadButtonPressed failed for gamepadId='$id' button='${button.name}' with code: $result"
                )
            } else {
                println(
                    "isGamepadButtonPressed failed for gamepadId='$id' button='${button.name}' with code: $result"
                )
                false
            }
        }
    }

    actual fun getStringProperty(entityHandle: Long, label: String): String? {
        val world = worldHandle ?: return null
        memScoped {
            val output = alloc<CPointerVar<ByteVar>>()

            val result = dropbear_get_string_property(
                world.reinterpret(),
                entityHandle,
                label,
                output.ptr
            )

            if (result == 0) {
                val string = output.value?.toKString()
                return string
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getStringProperty [$label] failed with code: $result")
                } else {
                    println("getStringProperty [$label] failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getIntProperty(entityHandle: Long, label: String): Int? {
        val world = worldHandle ?: return null
        memScoped {
            val output = alloc<IntVar>()

            val result = dropbear_get_int_property(
                world.reinterpret(),
                entityHandle,
                label,
                output.ptr,
            )

            if (result == 0) {
                return output.value
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getIntProperty [$label] failed with code: $result")
                } else {
                    println("getIntProperty [$label] failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getLongProperty(entityHandle: Long, label: String): Long? {
        val world = worldHandle ?: return null
        memScoped {
            val output = alloc<LongVar>()

            val result = dropbear_get_long_property(
                world.reinterpret(),
                entityHandle,
                label,
                output.ptr
            )

            if (result == 0) {
                return output.value
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getLongProperty [$label] failed with code: $result")
                } else {
                    println("getLongProperty [$label] failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getFloatProperty(entityHandle: Long, label: String): Float? {
        val world = worldHandle ?: return null
        memScoped {
            val output = alloc<DoubleVar>()

            val result = dropbear_get_float_property(
                world.reinterpret(),
                entityHandle,
                label,
                output.ptr
            )

            if (result == 0) {
                return output.value.toFloat()
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getFloatProperty [$label] failed with code: $result")
                } else {
                    println("getFloatProperty [$label] failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getDoubleProperty(entityHandle: Long, label: String): Double? {
        val world = worldHandle ?: return null
        memScoped {
            val output = alloc<DoubleVar>()

            val result = dropbear_get_float_property(
                world.reinterpret(),
                entityHandle,
                label,
                output.ptr
            )

            if (result == 0) {
                return output.value
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getDoubleProperty [$label] failed with code: $result")
                } else {
                    println("getDoubleProperty [$label] failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getBoolProperty(entityHandle: Long, label: String): Boolean? {
        val world = worldHandle ?: return null
        memScoped {
            val output = alloc<IntVar>()

            val result = dropbear_get_bool_property(
                world.reinterpret(),
                entityHandle,
                label,
                output.ptr
            )

            if (result == 0) {
                return output.value != 0
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getBoolProperty [$label] failed with code: $result")
                } else {
                    println("getBoolProperty [$label] failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getVec3Property(entityHandle: Long, label: String): FloatArray? {
        val world = worldHandle ?: return null
        memScoped {
            val outVec = alloc<Vector3D>()

            val result = dropbear_get_vec3_property(
                world.reinterpret(),
                entityHandle,
                label,
                outVec.ptr
            )

            if (result == 0) {
                return floatArrayOf(outVec.x.toFloat(), outVec.y.toFloat(), outVec.z.toFloat())
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getVec3Property [$label] failed with code: $result")
                } else {
                    println("getVec3Property [$label] failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun setStringProperty(entityHandle: Long, label: String, value: String) {
        val world = worldHandle ?: return

        val result = dropbear_set_string_property(
            world.reinterpret(),
            entityHandle,
            label,
            value
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setStringProperty [$label] failed with code: $result")
            } else {
                println("setStringProperty [$label] failed with code: $result")
            }
        }
    }

    actual fun setIntProperty(entityHandle: Long, label: String, value: Int) {
        val world = worldHandle ?: return

        val result = dropbear_set_int_property(
            world.reinterpret(),
            entityHandle,
            label,
            value
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setIntProperty [$label] failed with code: $result")
            } else {
                println("setIntProperty [$label] failed with code: $result")
            }
        }
    }

    actual fun setLongProperty(entityHandle: Long, label: String, value: Long) {
        val world = worldHandle ?: return

        val result = dropbear_set_long_property(
            world.reinterpret(),
            entityHandle,
            label,
            value
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setLongProperty [$label] failed with code: $result")
            } else {
                println("setLongProperty [$label] failed with code: $result")
            }
        }
    }

    actual fun setFloatProperty(entityHandle: Long, label: String, value: Double) {
        val world = worldHandle ?: return

        val result = dropbear_set_float_property(
            world.reinterpret(),
            entityHandle,
            label,
            value
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setFloatProperty [$label] failed with code: $result")
            } else {
                println("setFloatProperty [$label] failed with code: $result")
            }
        }
    }

    actual fun setBoolProperty(entityHandle: Long, label: String, value: Boolean) {
        val world = worldHandle ?: return
        val intValue = if (value) 1 else 0

        val result = dropbear_set_bool_property(
            world.reinterpret(),
            entityHandle,
            label,
            intValue
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setBoolProperty [$label] failed with code: $result")
            } else {
                println("setBoolProperty [$label] failed with code: $result")
            }
        }
    }

    actual fun setVec3Property(entityHandle: Long, label: String, value: FloatArray) {
        val world = worldHandle ?: return

        if (value.size < 3) {
            if (exceptionOnError) {
                throw DropbearNativeException("setVec3Property: FloatArray must have at least 3 elements")
            } else {
                println("setVec3Property: FloatArray must have at least 3 elements")
                return
            }
        }

        memScoped {
            val vec = cValue<Vector3D> {
                x = value[0].toDouble()
                y = value[1].toDouble()
                z = value[2].toDouble()
            }

            val result = dropbear_set_vec3_property(
                world.reinterpret(),
                entityHandle,
                label,
                vec
            )

            if (result != 0) {
                if (exceptionOnError) {
                    throw DropbearNativeException("setVec3Property [$label] failed with code: $result")
                } else {
                    println("setVec3Property [$label] failed with code: $result")
                }
            }
        }
    }

    actual fun getCamera(label: String): Camera? {
        val world = worldHandle ?: return null
        memScoped {
            val outCamera = alloc<NativeCamera>()

            val result = dropbear_get_camera(
                world.reinterpret(),
                label,
                outCamera.ptr
            )

            if (result == 0) {
                return Camera(
                    label = outCamera.label?.toKString() ?: "",
                    id = EntityId(outCamera.entity_id),
                    eye = com.dropbear.math.Vector3D(
                        outCamera.eye.x,
                        outCamera.eye.y,
                        outCamera.eye.z
                    ),
                    target = com.dropbear.math.Vector3D(
                        outCamera.target.x,
                        outCamera.target.y,
                        outCamera.target.z
                    ),
                    up = com.dropbear.math.Vector3D(
                        outCamera.up.x,
                        outCamera.up.y,
                        outCamera.up.z
                    ),
                    aspect = outCamera.aspect,
                    fov_y = outCamera.fov_y,
                    znear = outCamera.znear,
                    zfar = outCamera.zfar,
                    yaw = outCamera.yaw,
                    pitch = outCamera.pitch,
                    speed = outCamera.speed,
                    sensitivity = outCamera.sensitivity
                )
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getCamera failed with code: $result")
                } else {
                    println("getCamera failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getAttachedCamera(entityId: EntityId): Camera? {
        val world = worldHandle ?: return null
        memScoped {
            val outCamera = alloc<NativeCamera>()

            val result = dropbear_get_attached_camera(
                world.reinterpret(),
                entityId.id,
                outCamera.ptr
            )

            if (result == 0) {
                return Camera(
                    label = outCamera.label?.toKString() ?: "",
                    id = EntityId(outCamera.entity_id),
                    eye = com.dropbear.math.Vector3D(
                        outCamera.eye.x,
                        outCamera.eye.y,
                        outCamera.eye.z
                    ),
                    target = com.dropbear.math.Vector3D(
                        outCamera.target.x,
                        outCamera.target.y,
                        outCamera.target.z
                    ),
                    up = com.dropbear.math.Vector3D(
                        outCamera.up.x,
                        outCamera.up.y,
                        outCamera.up.z
                    ),
                    aspect = outCamera.aspect,
                    fov_y = outCamera.fov_y,
                    znear = outCamera.znear,
                    zfar = outCamera.zfar,
                    yaw = outCamera.yaw,
                    pitch = outCamera.pitch,
                    speed = outCamera.speed,
                    sensitivity = outCamera.sensitivity
                )
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getAttachedCamera failed with code: $result")
                } else {
                    println("getAttachedCamera failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun setCamera(camera: Camera) {
        val world = worldHandle ?: return
        memScoped {
            val nativeCamera = cValue<NativeCamera> {
                label = camera.label.cstr.ptr
                entity_id = camera.id.id

                eye.x = camera.eye.x
                eye.y = camera.eye.y
                eye.z = camera.eye.z

                target.x = camera.target.x
                target.y = camera.target.y
                target.z = camera.target.z

                up.x = camera.up.x
                up.y = camera.up.y
                up.z = camera.up.z

                aspect = camera.aspect
                fov_y = camera.fov_y
                znear = camera.znear
                zfar = camera.zfar

                yaw = camera.yaw
                pitch = camera.pitch
                speed = camera.speed
                sensitivity = camera.sensitivity
            }

            val result = dropbear_set_camera(
                world.reinterpret(),
                nativeCamera
            )

            if (result != 0) {
                if (exceptionOnError) {
                    throw DropbearNativeException("setCamera failed with code: $result")
                } else {
                    println("setCamera failed with code: $result")
                }
            }
        }
    }

    actual fun getModel(entityHandle: Long): Long? {
        val world = worldHandle ?: return null
        val asset = assetHandle ?: return null
        memScoped {
            val outModel = alloc<LongVar>()
            val result = dropbear_get_model(
                world.reinterpret(),
                asset.reinterpret(),
                entityHandle,
                outModel.ptr
            )
            return if (result == 0) outModel.value else if (exceptionOnError) throw DropbearNativeException("getModel failed with code: $result") else null
        }
    }

    actual fun setModel(entityHandle: Long, modelHandle: Long) {
        val world = worldHandle ?: return
        val asset = assetHandle ?: return

        val result = dropbear_set_model(
            world.reinterpret(),
            asset.reinterpret(),
            entityHandle,
            modelHandle
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setModel failed with code: $result")
            } else {
                println("setModel failed with code: $result")
            }
        }
    }

    actual fun getTexture(entityHandle: Long, name: String): Long? {
        val world = worldHandle ?: return null
        val asset = assetHandle ?: return null
        memScoped {
            val outTexture = alloc<LongVar>()
            val result = dropbear_get_texture(
                world.reinterpret(),
                asset.reinterpret(),
                entityHandle,
                name,
                outTexture.ptr
            )
            return if (result == 0) outTexture.value else if (exceptionOnError) throw DropbearNativeException("getTexture failed with code: $result") else null
        }
    }

    actual fun isUsingModel(entityHandle: Long, modelHandle: Long): Boolean {
        val world = worldHandle ?: return false
        memScoped {
            val outUsing = alloc<IntVar>()
            val result = dropbear_is_using_model(
                world.reinterpret(),
                entityHandle,
                modelHandle,
                outUsing.ptr
            )
            return if (result == 0) outUsing.value != 0 else false
        }
    }

    actual fun isUsingTexture(entityHandle: Long, textureHandle: Long): Boolean {
        val world = worldHandle ?: return false
        memScoped {
            val outUsing = alloc<IntVar>()
            val result = dropbear_is_using_texture(
                world.reinterpret(),
                entityHandle,
                textureHandle,
                outUsing.ptr
            )
            return if (result == 0) outUsing.value != 0 else false
        }
    }

    actual fun getAsset(eucaURI: String): Long? {
        val asset = assetHandle ?: return null
        memScoped {
            val outAsset = alloc<LongVar>()
            val result = dropbear_get_asset(
                asset.reinterpret(),
                eucaURI,
                outAsset.ptr
            )
            return if (result == 0) outAsset.value else null
        }
    }

    actual fun isModelHandle(id: Long): Boolean {
        val asset = assetHandle ?: return false
        memScoped {
            val outIsModel = alloc<IntVar>()
            val result = dropbear_is_model_handle(
                asset.reinterpret(),
                id,
                outIsModel.ptr
            )
            return if (result == 0) outIsModel.value != 0 else false
        }
    }

    actual fun isTextureHandle(id: Long): Boolean {
        val asset = assetHandle ?: return false
        memScoped {
            val outIsTexture = alloc<IntVar>()
            val result = dropbear_is_texture_handle(
                asset.reinterpret(),
                id,
                outIsTexture.ptr
            )
            return if (result == 0) outIsTexture.value != 0 else false
        }
    }

    actual fun setTextureOverride(entityHandle: Long, oldMaterialName: String, newTextureHandle: TextureHandle) {
        val world = worldHandle ?: return
        val asset = assetHandle ?: return

        val result = dropbear_set_texture(
            world.reinterpret(),
            asset.reinterpret(),
            entityHandle,
            oldMaterialName,
            newTextureHandle.raw()
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("setTextureOverride failed with code: $result")
            } else {
                println("setTextureOverride failed with code: $result")
            }
        }
    }

    actual fun getTextureName(textureHandle: Long): String? {
        val asset = assetHandle ?: return null
        memScoped {
            val outName = alloc<CPointerVar<ByteVar>>()
            val result = dropbear_get_texture_name(
                asset.reinterpret(),
                textureHandle,
                outName.ptr
            )
            return if (result == 0) outName.value?.toKString() else if (exceptionOnError) throw DropbearNativeException("getTextureName failed with code: $result") else null
        }
    }

    actual fun getAllTextures(entityHandle: Long): Array<String> {
        val world = worldHandle ?: return emptyArray()
        val asset = assetHandle ?: return emptyArray()
        memScoped {
            val outTextures = alloc<CPointerVar<CPointerVar<ByteVar>>>()
            val outCount = alloc<ULongVar>()

            val result = dropbear_get_all_textures(
                asset.reinterpret(),
                world.reinterpret(),
                entityHandle,
                outTextures.ptr,
                outCount.ptr
            )

            if (result == 0) {
                val count = outCount.value.toInt()
                val textureArray = Array(count) { i ->
                    outTextures.value!![i]?.toKString() ?: ""
                }
                return textureArray
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getAllTextures failed with code: $result")
                } else {
                    println("getAllTextures failed with code: $result")
                    return emptyArray()
                }
            }
        }
    }

    actual fun getChildren(entityId: EntityId): Array<EntityRef>? {
        val world = worldHandle ?: return null
        memScoped {
            val outChildren = alloc<CPointerVar<LongVar>>()
            val outCount = alloc<ULongVar>()

            val result = dropbear_get_children(
                world.reinterpret(),
                entityId.id,
                outChildren.ptr,
                outCount.ptr
            )

            if (result == 0) {
                val count = outCount.value.toInt()
                val childArray = Array(count) { i ->
                    EntityRef(EntityId(outChildren.value!![i]))
                }
                return childArray
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getChildren failed with code: $result")
                } else {
                    println("getChildren failed with code: $result")
                    return null
                }
            }
        }
    }

    actual fun getChildByLabel(entityId: EntityId, label: String): EntityRef? {
        val world = worldHandle ?: return null
        memScoped {
            val outChild = alloc<LongVar>()
            val result = dropbear_get_child_by_label(
                world.reinterpret(),
                entityId.id,
                label,
                outChild.ptr
            )
            return if (result == 0) EntityRef(EntityId(outChild.value)) else if (exceptionOnError) throw DropbearNativeException("getChildByLabel failed with code: $result") else null
        }
    }

    actual fun getParent(entityId: EntityId): EntityRef? {
        val world = worldHandle ?: return null
        memScoped {
            val outParent = alloc<LongVar>()
            val result = dropbear_get_parent(
                world.reinterpret(),
                entityId.id,
                outParent.ptr
            )
            return if (result == 0) EntityRef(EntityId(outParent.value)) else if (exceptionOnError) throw DropbearNativeException("getParent failed with code: $result") else null
        }
    }

    actual fun quit() {
        val command = commandBufferHandle ?: if (exceptionOnError) throw DropbearNativeException("Unable to quit: graphicsHandle does not exist") else return
        dropbear_quit(command.reinterpret())
    }

    actual fun loadSceneAsync(sceneName: String): SceneLoadHandle? {
        val sceneLoader = sceneLoaderHandle ?: if (exceptionOnError) throw DropbearNativeException("sceneLoaderHandle is empty") else return null
        val commandBuffer = commandBufferHandle ?: if (exceptionOnError) throw DropbearNativeException("commandBufferHandle is empty") else return null
        memScoped {
            val outHandle = alloc<com.dropbear.ffi.generated.SceneLoadHandle>()
            val result = dropbear_load_scene_async_1(
                command_ptr = commandBuffer.reinterpret(),
                scene_loader_ptr = sceneLoader.reinterpret(),
                name = sceneName,
                sceneLoadHandle = outHandle.ptr
            )

            if (result != 0) {
                if (exceptionOnError) {
                    throw DropbearNativeException("loadSceneAsync failed with code: $result")
                } else {
                    println("loadSceneAsync failed with code: $result")
                }
            }

            return SceneLoadHandle(
                id = outHandle.id,
                sceneName = outHandle.name?.toKString() ?: throw Exception("loadSceneAsync failed: sceneName is null"),
                native = this@NativeEngine,
            )
        }
    }

    actual fun loadSceneAsync(sceneName: String, loadingScene: String): SceneLoadHandle? {
        val sceneLoader = sceneLoaderHandle ?: if (exceptionOnError) throw DropbearNativeException("sceneLoaderHandle is empty") else return null
        val commandBuffer = commandBufferHandle ?: if (exceptionOnError) throw DropbearNativeException("commandBufferHandle is empty") else return null
        memScoped {
            val outHandle = alloc<com.dropbear.ffi.generated.SceneLoadHandle>()
            val result = dropbear_load_scene_async_2(
                command_ptr = commandBuffer.reinterpret(),
                scene_loader_ptr = sceneLoader.reinterpret(),
                name = sceneName,
                loadingScene = loadingScene,
                sceneLoadHandle = outHandle.ptr
            )

            if (result != 0) {
                if (exceptionOnError) {
                    throw DropbearNativeException("loadSceneAsync with loading scene failed with code: $result")
                } else {
                    println("loadSceneAsync with loading scene failed with code: $result")
                }
            }

            return SceneLoadHandle(
                id = outHandle.id,
                sceneName = outHandle.name?.toKString() ?: throw Exception("loadSceneAsync with loading scene failed: sceneName is null"),
                native = this@NativeEngine,
            )
        }
    }

    actual fun switchToSceneAsync(sceneLoadHandle: SceneLoadHandle) {
        val command = commandBufferHandle ?: if (exceptionOnError) throw DropbearNativeException("commandBufferHandle is empty") else return
        memScoped {
            val nativeHandle = alloc<com.dropbear.ffi.generated.SceneLoadHandle>()
            nativeHandle.id = sceneLoadHandle.id
            nativeHandle.name = sceneLoadHandle.sceneName.cstr.ptr

            val result = dropbear_switch_to_scene_async(command.reinterpret(), nativeHandle.readValue())

            if (result == -10) {
                throw PrematureSceneSwitchException("Attempted to switch to scene before it finished loading")
            }

            if (result != 0) {
                if (exceptionOnError) {
                    throw DropbearNativeException("switchToSceneAsync failed with code: $result")
                } else {
                    println("switchToSceneAsync failed with code: $result")
                }
            }
        }
    }

    actual fun switchToSceneImmediate(sceneName: String) {
        val command = commandBufferHandle ?: if (exceptionOnError) throw DropbearNativeException("commandBufferHandle is empty") else return
        val result = dropbear_switch_to_scene_immediate(
            command_ptr = command.reinterpret(),
            name = sceneName,
        )

        if (result != 0) {
            if (exceptionOnError) {
                throw DropbearNativeException("switchToSceneImmediate failed with code: $result")
            } else {
                println("switchToSceneImmediate failed with code: $result")
            }
        }
    }

    actual fun getSceneLoadProgress(sceneLoadHandle: SceneLoadHandle): Progress {
        val sceneLoader = sceneLoaderHandle ?: if (exceptionOnError) throw DropbearNativeException("sceneLoaderHandle is empty") else return Progress.nothing("Error: commandBufferHandle is empty")
        memScoped {
            val nativeHandle = alloc<com.dropbear.ffi.generated.SceneLoadHandle>()
            nativeHandle.id = sceneLoadHandle.id
            nativeHandle.name = sceneLoadHandle.sceneName.cstr.ptr

            val outProgress = alloc<com.dropbear.ffi.generated.Progress>()

            val result = dropbear_get_scene_load_progress(
                scene_loader_ptr = sceneLoader.reinterpret(),
                handle = nativeHandle.readValue(),
                progress = outProgress.ptr
            )

            if (result != 0) {
                if (exceptionOnError) {
                    throw DropbearNativeException("getSceneLoadProgress failed with code: $result")
                } else {
                    println("getSceneLoadProgress failed with code: $result")
                }
            }

            val progress = Progress(outProgress.current, outProgress.total, outProgress.message?.toKString());

            return progress
        }
    }

    actual fun getSceneLoadStatus(sceneLoadHandle: SceneLoadHandle): SceneLoadStatus {
        val sceneLoader = sceneLoaderHandle ?: if (exceptionOnError) throw DropbearNativeException("sceneLoaderHandle is empty") else return SceneLoadStatus.FAILED
        memScoped {
            val nativeHandle = alloc<com.dropbear.ffi.generated.SceneLoadHandle>()
            nativeHandle.id = sceneLoadHandle.id
            nativeHandle.name = sceneLoadHandle.sceneName.cstr.ptr

            val nativeStatus = alloc<SceneLoadResult.Var>()

            val result = dropbear_get_scene_load_status(
                scene_loader_ptr = sceneLoader.reinterpret(),
                handle = nativeHandle.readValue(),
                result = nativeStatus.ptr
            )
            if (result == 0) {
                return nativeStatus.value.fromNative()
            } else {
                if (exceptionOnError) {
                    throw DropbearNativeException("getSceneLoadStatus failed with code: $result")
                } else {
                    println("getSceneLoadStatus failed with code: $result")
                    return SceneLoadStatus.FAILED
                }
            }
        }
    }

    actual fun setPhysicsEnabled(entityId: Long, enabled: Boolean) {
        val pe = physicsEngineHandle ?: if (exceptionOnError) throw DropbearNativeException("Physics engine handle is null") else return

        val result = dropbear_set_physics_enabled(
            pe.reinterpret(),
            entityId,
            enabled
        )

        handleResult(result, "setPhysicsEnabled")
    }

    actual fun isPhysicsEnabled(entityId: Long): Boolean {
        val pe = physicsEngineHandle ?: if (exceptionOnError) throw DropbearNativeException("Physics engine handle is null") else return false

        memScoped {
            val outEnabled = alloc<BooleanVar>()

            val result = dropbear_is_physics_enabled(
                pe.reinterpret(),
                entityId,
                outEnabled.ptr
            )

            if (result != 0) {
                handleResult(result, "isPhysicsEnabled")
                return false
            }

            return outEnabled.value
        }
    }

    actual fun getRigidBody(entityId: Long): RigidBody? {
        val pe = physicsEngineHandle ?: if (exceptionOnError) throw DropbearNativeException("Physics engine handle is null") else return null

        memScoped {
            val outRigidBody = alloc<com.dropbear.ffi.generated.RigidBody>()

            val result = dropbear_get_rigidbody(
                pe.reinterpret(),
                entityId,
                outRigidBody.ptr
            )

            if (result != 0) {
                handleResult(result, "getRigidBody")
                return null
            }

            return outRigidBody.toKotlin(this@NativeEngine)
        }
    }

    actual fun getAllColliders(entityId: Long): List<Collider> {
        val pe = physicsEngineHandle ?: if (exceptionOnError) throw DropbearNativeException("Physics engine handle is null") else return emptyList()

        memScoped {
            val outCollidersPtr = alloc<CPointerVar<com.dropbear.ffi.generated.Collider>>()
            val outCount = alloc<UIntVar>()

            val result = dropbear_get_all_colliders(
                pe.reinterpret(),
                entityId,
                outCollidersPtr.ptr,
                outCount.ptr
            )

            if (result != 0) {
                handleResult(result, "getAllColliders")
                return emptyList()
            }

            val count = outCount.value.toInt()
            val cArray = outCollidersPtr.value

            if (cArray == null || count == 0) {
                return emptyList()
            }

            val list = ArrayList<Collider>(count)
            for (i in 0 until count) {
                val cCollider = cArray[i]
                list.add(cCollider.toKotlin(this@NativeEngine))
            }

            dropbear_free_colliders(cArray, outCount.value)

            return list
        }
    }

    actual fun applyImpulse(index: Index, x: Double, y: Double, z: Double) {
        val pe = physicsEngineHandle ?: return

        memScoped {
            val cIndex = alloc<com.dropbear.ffi.generated.Index>()
            cIndex.index = index.index
            cIndex.generation = index.generation

            val cImpulse = alloc<com.dropbear.ffi.generated.Vector3D>()
            cImpulse.x = x
            cImpulse.y = y
            cImpulse.z = z

            dropbear_apply_impulse(
                pe.reinterpret(),
                cIndex.readValue(),
                cImpulse.readValue()
            )
        }
    }

    actual fun applyTorqueImpulse(index: Index, x: Double, y: Double, z: Double) {
        val pe = physicsEngineHandle ?: return

        memScoped {
            val cIndex = alloc<com.dropbear.ffi.generated.Index>()
            cIndex.index = index.index
            cIndex.generation = index.generation

            val cTorque = alloc<com.dropbear.ffi.generated.Vector3D>()
            cTorque.x = x
            cTorque.y = y
            cTorque.z = z

            dropbear_apply_torque_impulse(
                pe.reinterpret(),
                cIndex.readValue(),
                cTorque.readValue()
            )
        }
    }

    actual fun setRigidbody(rigidBody: RigidBody) {
        val pe = physicsEngineHandle ?: if (exceptionOnError) throw DropbearNativeException("Handle null") else return

        memScoped {
            val cBody = alloc<com.dropbear.ffi.generated.RigidBody>()
            rigidBody.populateCStruct(cBody)

            val result = dropbear_set_rigidbody(
                pe.reinterpret(),
                cBody.readValue()
            )
            handleResult(result, "setRigidbody")
        }
    }

    actual fun getChildColliders(index: Index): List<Collider> {
        val pe = physicsEngineHandle ?: if (exceptionOnError) throw DropbearNativeException("Handle null") else return emptyList()

        memScoped {
            val cIndex = alloc<com.dropbear.ffi.generated.Index>()
            cIndex.index = index.index
            cIndex.generation = index.generation

            val outCollidersPtr = alloc<CPointerVar<com.dropbear.ffi.generated.Collider>>()
            val outCount = alloc<UIntVar>()

            val result = dropbear_get_child_colliders(
                pe.reinterpret(),
                cIndex.readValue(),
                outCollidersPtr.ptr,
                outCount.ptr
            )

            if (result != 0) {
                handleResult(result, "getChildColliders")
                return emptyList()
            }

            val count = outCount.value.toInt()
            val cArray = outCollidersPtr.value

            if (cArray == null || count == 0) return emptyList()

            val list = ArrayList<Collider>(count)
            for (i in 0 until count) {
                list.add(cArray[i].toKotlin(this@NativeEngine))
            }

            dropbear_free_colliders(cArray, outCount.value)
            return list
        }
    }

    actual fun setCollider(collider: Collider) {
        val pe = physicsEngineHandle ?: if (exceptionOnError) throw DropbearNativeException("Handle null") else return

        memScoped {
            val cCollider = alloc<com.dropbear.ffi.generated.Collider>()
            collider.populateCStruct(cCollider)

            val result = dropbear_set_collider(
                pe.reinterpret(),
                cCollider.readValue()
            )
            handleResult(result, "setCollider")
        }
    }
}

private fun handleResult(result: Int, funcName: String) {
    if (result != 0) {
        val msg = "$funcName failed with code: $result"
        if (exceptionOnError) {
            throw DropbearNativeException(msg)
        } else {
            Logger.error(msg)
        }
    }
}