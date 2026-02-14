package com.dropbear.ui.styling.fonts

sealed interface Family {
    data class Name(val value: String) : Family
    data object Serif : Family
    data object SansSerif : Family
    data object Cursive : Family
    data object Fantasy : Family
    data object Monospace : Family
}