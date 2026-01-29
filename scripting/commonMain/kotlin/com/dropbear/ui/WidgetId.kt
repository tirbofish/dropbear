package com.dropbear.ui

import com.dropbear.utils.ID

class WidgetId(val id: Long) : ID(id) {
    constructor(id: String) : this(id.hashCode().toLong())
}