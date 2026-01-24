package com.dropbear.logging

expect class SocketWriter(): LogWriter {
    override fun log(
        level: LogLevel,
        target: String,
        message: String,
        file: String?,
        line: Int?
    )
}
