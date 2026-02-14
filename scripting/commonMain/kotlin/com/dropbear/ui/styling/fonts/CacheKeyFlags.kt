package com.dropbear.ui.styling.fonts

import kotlin.jvm.JvmInline

@JvmInline
value class CacheKeyFlags(val bits: Int = 0) {
    operator fun plus(flag: CacheKeyFlags): CacheKeyFlags =
        CacheKeyFlags(bits or flag.bits)

    operator fun contains(flag: CacheKeyFlags): Boolean =
        (bits and flag.bits) != 0

    companion object {
        val NONE = CacheKeyFlags(0)
    }
}