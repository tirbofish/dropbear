package com.dropbear.ui.styling.fonts

class FontFeatures(val features: MutableList<FontFeature> = mutableListOf()) {
    fun set(tag: FeatureTag, value: UInt) {
        features.add(FontFeature(tag, value))
    }

    fun enable(tag: FeatureTag) {
        set(tag, 1u)
    }

    fun disable(tag: FeatureTag) {
        set(tag, 0u)
    }
}