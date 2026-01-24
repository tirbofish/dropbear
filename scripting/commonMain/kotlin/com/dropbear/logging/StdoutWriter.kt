package com.dropbear.logging

import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Clock

class StdoutWriter: LogWriter {
    private val reset = "\u001B[0m"

    override fun log(
        level: LogLevel,
        target: String,
        message: String,
        file: String?,
        line: Int?
    ) {
        val now = Clock.System.now()
        val timeZone = TimeZone.currentSystemDefault()
        val timestamp = now.toLocalDateTime(timeZone)
        val location = if (file != null && line != null) "[$file:$line] " else ""
        when (level) {
            LogLevel.ERROR -> println("${level.ansi}[$timestamp] [$level] $location[$target] $message$reset")
            LogLevel.WARN -> println("${level.ansi}[$timestamp] [$level] $location[$target] $message$reset")
            else -> println("[$timestamp] [${level.ansi}$level$reset] $location[$target] $message")
        }
    }
}