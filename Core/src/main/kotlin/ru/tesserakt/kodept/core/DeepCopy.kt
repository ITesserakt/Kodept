package ru.tesserakt.kodept.core

interface DeepCopyable<out T : DeepCopyable<T>> {
    fun deepCopy(): T
}