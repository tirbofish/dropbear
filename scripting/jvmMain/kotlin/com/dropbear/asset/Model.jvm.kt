package com.dropbear.asset

import com.dropbear.DropbearEngine
import com.dropbear.asset.model.Animation
import com.dropbear.asset.model.Material
import com.dropbear.asset.model.Mesh
import com.dropbear.asset.model.Node
import com.dropbear.asset.model.Skin

actual fun Model.getLabel(): String {
    return ModelNative.getLabel(DropbearEngine.native.assetHandle, label)
}

actual fun Model.getMeshes(): List<Mesh> {
    return ModelNative.getMeshes(DropbearEngine.native.assetHandle, this.id).toList()
}

actual fun Model.getMaterials(): List<Material> {
    return ModelNative.getMaterials(DropbearEngine.native.assetHandle, this.id).toList()
}

actual fun Model.getSkins(): List<Skin> {
    return ModelNative.getSkins(DropbearEngine.native.assetHandle, this.id).toList()
}

actual fun Model.getAnimations(): List<Animation> {
    return ModelNative.getAnimations(DropbearEngine.native.assetHandle, this.id).toList()
}

actual fun Model.getNodes(): List<Node> {
    return ModelNative.getNodes(DropbearEngine.native.assetHandle, this.id).toList()
}