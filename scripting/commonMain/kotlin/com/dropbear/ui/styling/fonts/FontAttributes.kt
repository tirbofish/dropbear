package com.dropbear.ui.styling.fonts

import com.dropbear.utils.Colour

class FontAttributes(
    var colourOptions: Colour? = null,
    var family: Family = Family.Name("default"),
    var stretch: Stretch = Stretch.Normal,
    var style: FontStyle = FontStyle.Normal,
    var weight: FontWeight = FontWeight.NORMAL,
    var metadata: UInt = 0u,
    var cacheKeyFlags: CacheKeyFlags = CacheKeyFlags.NONE,
    var metricsOptions: CacheMetrics? = null,
    var letterSpacingOptions: Double? = null,
    var fontFeatures: FontFeatures = FontFeatures(),
)