package com.dropbear.physics

/**
 * Determines a index and a generation. This fixes the famous ABA problem.
 *
 * @see <a href="https://en.wikipedia.org/wiki/ABA_problem">ABA Problem - Wikipedia
 */
data class Index(val index: UInt, val generation: UInt)