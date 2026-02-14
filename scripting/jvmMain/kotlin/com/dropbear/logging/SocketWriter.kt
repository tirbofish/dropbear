package com.dropbear.logging

import java.net.Socket
import java.io.PrintWriter
import java.util.concurrent.Executors
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Clock

actual class SocketWriter actual constructor() : LogWriter {
    private val executor = Executors.newSingleThreadExecutor()
    private var writer: PrintWriter? = null
    
    init {
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            val stackTrace = java.io.StringWriter().apply {
                throwable.printStackTrace(java.io.PrintWriter(this))
            }.toString()
            
            log(LogLevel.ERROR, "UncaughtException", "Exception in thread \"${thread.name}\": $throwable\n$stackTrace", null, null)
        }

        executor.submit {
            try {
                // Connect to the editor console
                val socket = Socket("127.0.0.1", 56624)
                writer = PrintWriter(socket.getOutputStream(), true)
            } catch (e: Exception) {
                // Silently fail or log to stderr if connection fails
                System.err.println("Failed to connect to logging socket: ${e.message}")
            }
        }
    }

    actual override fun log(
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
        
        val formatted = "[$timestamp] [$level] $location[$target] $message"
        
        executor.submit {
            try {
                writer?.println(formatted)
            } catch (e: Exception) {
                // Ignore write errors to avoid crashing
            }
        }
    }
}
