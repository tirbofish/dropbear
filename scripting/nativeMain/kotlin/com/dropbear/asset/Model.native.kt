package com.dropbear.asset

import com.dropbear.asset.model.Animation
import com.dropbear.asset.model.Material
import com.dropbear.asset.model.Mesh
import com.dropbear.asset.model.Node
import com.dropbear.asset.model.Skin

actual fun Model.getLabel(): String {
    TODO("Not yet implemented")
}

actual fun Model.getMeshes(): List<Mesh> {
    return emptyList()
}

actual fun Model.getMaterials(): List<Material> {
    return emptyList()
}

actual fun Model.getSkins(): List<Skin> {
    return emptyList()
}

actual fun Model.getAnimations(): List<Animation> {
    return emptyList()
}

actual fun Model.getNodes(): List<Node> {
    return emptyList()
}
