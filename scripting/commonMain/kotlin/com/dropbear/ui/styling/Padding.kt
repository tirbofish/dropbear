package com.dropbear.ui.styling

import com.dropbear.math.Vector2d

data class Padding(
    var left: Double,
    var right: Double,
    var top: Double,
    var bottom: Double,
) {
    companion object {
        fun zero() = Padding.all(0.0)

        fun all(value: Double) = Padding(value, value, value, value)
        fun balanced(horizontal: Double, vertical: Double) = Padding(horizontal, horizontal, vertical, vertical)
        fun horizontal(horizontal: Double) = Padding(horizontal, horizontal, 0.0, 0.0)
        fun vertical(vertical: Double) = Padding(0.0, 0.0, vertical, vertical)
    }

    fun offset(): Vector2d = Vector2d(left, top)

    override fun toString() = "($left, $right, $top, $bottom)"
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Padding) return false
        if (left != other.left) return false
        if (right != other.right) return false
        if (top != other.top) return false
        if (bottom != other.bottom) return false
        return true
    }
    override fun hashCode(): Int {
        var result = left.hashCode()
        result = 31 * result + right.hashCode()
        result = 31 * result + top.hashCode()
        result = 31 * result + bottom.hashCode()
        return result
    }
}