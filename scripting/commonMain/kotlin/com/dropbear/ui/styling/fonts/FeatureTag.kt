package com.dropbear.ui.styling.fonts

import kotlin.jvm.JvmInline

@JvmInline
value class FeatureTag(val tag: ByteArray) {
    init {
        require(tag.size == 4) { "Tag must be exactly 4 bytes" }
    }

    companion object {
        /** Kerning adjusts spacing between specific character pairs */
        val KERNING = FeatureTag("kern".encodeToByteArray())
        /** Standard ligatures (fi, fl, etc.) */
        val STANDARD_LIGATURES = FeatureTag("liga".encodeToByteArray())
        /** Contextual ligatures (context-dependent ligatures) */
        val CONTEXTUAL_LIGATURES = FeatureTag("clig".encodeToByteArray())
        /** Contextual alternates (glyph substitutions based on context) */
        val CONTEXTUAL_ALTERNATES = FeatureTag("calt".encodeToByteArray())
        /** Discretionary ligatures (optional stylistic ligatures) */
        val DISCRETIONARY_LIGATURES = FeatureTag("dlig".encodeToByteArray())
        /** Small caps (lowercase to small capitals) */
        val SMALL_CAPS = FeatureTag("smcp".encodeToByteArray())
        /** All small caps (uppercase and lowercase to small capitals) */
        val ALL_SMALL_CAPS = FeatureTag("c2sc".encodeToByteArray())
        /** Stylistic Set 1 (font-specific alternate glyphs) */
        val STYLISTIC_SET_1 = FeatureTag("ss01".encodeToByteArray())
        /** Stylistic Set 2 (font-specific alternate glyphs) */
        val STYLISTIC_SET_2 = FeatureTag("ss02".encodeToByteArray())
    }

    fun asBytes(): ByteArray = tag
}