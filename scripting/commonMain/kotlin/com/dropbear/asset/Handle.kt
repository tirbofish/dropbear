package com.dropbear.asset

import com.dropbear.utils.ID

/**
 * Describes a handle of an asset, or anything really.
 *
 * Aims to allow people to group up different handle types
 * into a list or a vector.
 *
 * All handles must be positive, non-zero values. If the id does not follow that rule, it is considered invalid.
 */
class Handle<T: AssetType>(private val id: Long): ID(id) {
    init {
        require(id > 0) { "Handle id must be a positive value. Got: $id" }
    }

    companion object {
        fun <T: AssetType> invalid() = Handle<T>(0)
    }

    fun raw(): Long {
        return id
    }
}

typealias TextureHandle = Handle<Texture>
typealias ModelHandle = Handle<Model>