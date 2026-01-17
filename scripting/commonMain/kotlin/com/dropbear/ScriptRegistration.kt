package com.dropbear

import com.dropbear.ecs.System

/**
 * Internal data class to register scripts with tags.
 */
data class ScriptRegistration(
    val tags: List<String>,
    val script: System
)