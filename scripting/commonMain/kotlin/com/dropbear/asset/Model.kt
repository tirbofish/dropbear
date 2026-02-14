package com.dropbear.asset

import com.dropbear.asset.model.Mesh
import com.dropbear.asset.model.Material
import com.dropbear.asset.model.Skin
import com.dropbear.asset.model.Animation
import com.dropbear.asset.model.Node

class Model(override val id: Long): AssetType(id) {
    val label: String
        get() = getLabel()

    val meshes: List<Mesh>
        get() = getMeshes()

    val materials: List<Material>
        get() = getMaterials()

    val skins: List<Skin>
        get() = getSkins()

    val animations: List<Animation>
        get() = getAnimations()

    val nodes: List<Node>
        get() = getNodes()
}

expect fun Model.getLabel(): String
expect fun Model.getMeshes(): List<Mesh>
expect fun Model.getMaterials(): List<Material>
expect fun Model.getSkins(): List<Skin>
expect fun Model.getAnimations(): List<Animation>
expect fun Model.getNodes(): List<Node>