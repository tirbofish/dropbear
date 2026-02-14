package com.dropbear.ui

typealias UIInstructionSet = List<UIInstruction>

class UIBuilder {
    val instructions: MutableList<UIInstruction> = mutableListOf()
    private var nextId = 0
    internal fun generateId(): Int = nextId++

    fun build(): UIInstructionSet = instructions.toList()
}

inline fun buildUI(block: UIBuilder.() -> Unit): UIInstructionSet {
    return UIBuilder().apply(block).build()
}

fun UIBuilder.add(instruction: UIInstruction) {
    instructions.add(instruction)
}

fun UIBuilder.add(instructionSet: UIInstructionSet) {
    instructions.addAll(instructionSet)
}

fun UIBuilder.add(block: UIBuilder.() -> Widget) = add(block().toInstruction())