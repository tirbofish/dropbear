package com.dropbear.utils

/**
 * Determines a range from a beginning to an end as determined through `start < end`.
 *
 * @property start The start value of a range
 * @property end The last value of a range.
 * The ending **does not** include the last value. The range class mathematically counts it to be `start < end`.
 * If you want to include a value (such that it is `start <= end`), you should use [RangeInclusive].
 */
class Range(
    var start: Double,
    var end: Double,
)