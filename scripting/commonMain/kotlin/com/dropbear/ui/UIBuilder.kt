package com.dropbear.ui

class UIBuilder {
    val instructions: MutableList<UIInstruction> = mutableListOf()
    private var nextId = 0
    internal fun generateId(): Int = nextId++

    fun build(): List<UIInstruction> = instructions.toList()
}

inline fun buildUI(block: UIBuilder.() -> Unit): List<UIInstruction> {
    return UIBuilder().apply(block).build()
}

fun UIBuilder.add(instruction: UIInstruction) {
    instructions.add(instruction)
}