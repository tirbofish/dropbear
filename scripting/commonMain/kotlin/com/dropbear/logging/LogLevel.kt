package com.dropbear.logging

enum class LogLevel(val ansi: String) {
    TRACE("\u001B[38;5;81m"),
    DEBUG("\u001B[38;5;220m"),
    INFO("\u001B[38;5;46m"),
    WARN("\u001B[38;5;214m"),
    ERROR("\u001B[31m"),
}