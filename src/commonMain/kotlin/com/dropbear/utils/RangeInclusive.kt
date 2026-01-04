package com.dropbear.utils

/**
 * Determines a range from a beginning to an end as determined through `start <= end`.
 *
 * @property start The start value of a range
 * @property end The last value of a range, inclusive.
 * The ending includes the last value. The range class mathematically counts it to be `start <= end`.
 * If you want to exclude the ending (such that it is `start < end`), you should use [Range].
 */
class RangeInclusive(
    var start: Double,
    var end: Double,
)