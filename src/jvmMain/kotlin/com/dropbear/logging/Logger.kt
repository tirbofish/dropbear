package com.dropbear.logging

internal actual fun getCallerInfo(): String {
    val stackTrace = Thread.currentThread().stackTrace
    val callerFrame = stackTrace.getOrNull(5) ?: return "unknown"

    val className = callerFrame.className.substringAfterLast('.')
    val methodName = callerFrame.methodName

    return "$className::$methodName"
}