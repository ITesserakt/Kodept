package ru.tesserakt.kodept.core

import io.kotest.assertions.withClue
import io.kotest.matchers.should
import io.kotest.matchers.shouldBe
import io.kotest.matchers.types.beOfType
import kotlin.reflect.KClass
import kotlin.reflect.KMutableProperty
import kotlin.reflect.KProperty
import kotlin.reflect.full.memberProperties
import kotlin.reflect.jvm.isAccessible

private fun KClass<*>.getAllInheritors(): List<KClass<*>> =
    listOf(this) + sealedSubclasses.flatMap { it.getAllInheritors() }

fun <T : AST.Node> compareNodesByFields(value: T, other: T, exclude: Set<KProperty<*>>) {
    val propsToCompare = value::class.memberProperties
        .onEach { it.isAccessible = true }
        .subtract(exclude)
        .sortedBy { it.name }

    value should beOfType(other::class)

    propsToCompare.forEach {
        val actual = it.call(value)
        val expected = it.call(other)

        withClue("Field `${it::class.qualifiedName}${it.name}` isn't equal between $value and $other") {
            if (actual is Iterable<*> && expected is Iterable<*>) {
                actual.zip(expected).forEach { (a, b) ->
                    a as AST.Node
                    b as AST.Node
                    compareNodesByFields(a, b, exclude)
                }
            } else if (actual is AST.Node && expected is AST.Node) {
                compareNodesByFields(actual, expected, exclude)
            } else {
                actual shouldBe expected
            }
        }
    }
}

private val defaultExcludes = (AST.Node::class.getAllInheritors().asSequence() + sequenceOf(
    AST.ResolvedReference::class,
    AST.ResolvedTypeReference::class,
    AST.Parameter::class
)).flatMap { it.memberProperties }
    .onEach { it.isAccessible = true }
    .filter { it.isLateinit || it.name == "id" || it is KMutableProperty<*> || it.name == "parent" || it.name == "rlt" }
    .toSet()

infix fun <T : AST.Node> T.structuralShouldBe(other: T) = compareNodesByFields(this, other, defaultExcludes)

infix fun <T : AST.Node> T.shouldBe(other: T) = structuralShouldBe(other)