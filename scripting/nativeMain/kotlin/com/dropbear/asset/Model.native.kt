@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.asset

import com.dropbear.DropbearEngine
import com.dropbear.asset.model.Animation
import com.dropbear.asset.model.Material
import com.dropbear.asset.model.Mesh
import com.dropbear.asset.model.Node
import com.dropbear.asset.model.Skin
import com.dropbear.ffi.generated.*
import kotlin.String
import kotlinx.cinterop.*

actual fun Model.getLabel(): String = memScoped {
    val assets = DropbearEngine.native.assetHandle ?: return@memScoped ""
    val out = alloc<CPointerVar<ByteVar>>()
    val rc = dropbear_asset_model_get_label(assets, id.toULong(), out.ptr)
    if (rc != 0) "" else out.value?.toKString() ?: ""
}

actual fun Model.getMeshes(): List<Mesh> = memScoped {
    TODO("tbc")
//
//    val result = dropbear_asset_model_get_meshes(
//
//    )
}
actual fun Model.getMaterials(): List<Material> = emptyList()
actual fun Model.getSkins(): List<Skin> = emptyList()
actual fun Model.getAnimations(): List<Animation> = emptyList()
actual fun Model.getNodes(): List<Node> = emptyList()
