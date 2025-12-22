package com.dropbear.logging

internal actual fun getCallerInfo(): String {
    return try {
        val stackTrace = Exception().stackTraceToString()

        stackTrace.lineSequence()
            .drop(2)
            .firstOrNull { line ->
                !line.contains("Logger") &&
                        !line.contains("getCallerInfo") &&
                        line.trimStart().startsWith("at ")
            }
            ?.let { line ->
                val functionPart = line
                    .substringAfter("at ", "")
                    .substringBefore(" (", "")
                    .trim()

                val parts = functionPart.split('.')
                if (parts.size >= 2) {
                    val className = parts[parts.lastIndex - 1]
                    val methodName = parts.last()
                    "$className::$methodName"
                } else {
                    "native::$functionPart"
                }
            }
            ?: "native::unknown"
    } catch (_: Throwable) {
        "native::unknown"
    }
}