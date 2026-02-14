package com.dropbear.logging

actual class SocketWriter actual constructor() : LogWriter {
    actual override fun log(
        level: LogLevel,
        target: String,
        message: String,
        file: String?,
        line: Int?
    ) {
        TODO("Not yet implemented")
    }

}
