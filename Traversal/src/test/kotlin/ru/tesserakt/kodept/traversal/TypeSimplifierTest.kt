package ru.tesserakt.kodept.traversal

import arrow.core.nonEmptyListOf
import io.kotest.assertions.arrow.core.shouldBeLeft
import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.mockk.clearAllMocks
import io.mockk.every
import io.mockk.mockk
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.error.Report

@OptIn(Internal::class)
class TypeSimplifierTest : StringSpec({
    lateinit var tupleRLT: RLT.TupleType
    lateinit var unionRLT: RLT.UnionType

    fun AST.UnionType.pushRLT() = with(unionRLT) { withRLT() }
    fun AST.TupleType.pushRLT() = with(tupleRLT) { withRLT() }

    beforeTest {
        tupleRLT = mockk {
            every { position } returns CodePoint(0, 0)
        }
        unionRLT = mockk {
            every { position } returns CodePoint(0, 0)
        }
    }

    with("TEST FILE" as Filename) {
        val transformer = TypeSimplifier

        "simple type should not change" {
            val type = AST.Type("A")
            unwrap { transformer.transform(type) }.toEither().shouldBeRight(type)
        }

        "Compound tuple types should not change" {
            val type = AST.TupleType(listOf("A", "B").map(AST::Type).map(AST::TypeReference)).pushRLT()
            unwrap { transformer.transform(type) }.toEither().shouldBeRight(type)
        }

        "Proper union types should not change" {
            val type = AST.UnionType(nonEmptyListOf("A", "B").map(AST::Type).map(AST::TypeReference)).pushRLT()
            unwrap { transformer.transform(type) }.toEither().shouldBeRight(type)
        }

        "Single type in tuple should be aligned with it" {
            val inner = AST.TupleType(listOf("A", "B").map(AST::Type).map(AST::TypeReference)).pushRLT()
            val type = AST.TupleType(listOf(inner).map(AST::TypeReference)).pushRLT()
            unwrap { transformer.transform(type) }.unwrap().shouldBeRight().second shouldBe inner
        }

        "Single union types should cause crash" {
            val union = AST.UnionType(nonEmptyListOf("A").map(AST::Type).map(AST::TypeReference)).pushRLT()
            unwrap { transformer.transform(union) }.toEither()
                .shouldBeLeft().head.severity shouldBe Report.Severity.CRASH
        }

        "Identical items in union should be aligned and flattened" {
            val a = AST.Type("A")
            val b = AST.TupleType(listOf("B", "C").map(AST::Type).map(AST::TypeReference)).pushRLT()
            val c = AST.UnionType(nonEmptyListOf(AST.Type("C"), a).map(AST::TypeReference)).pushRLT()
            val type = AST.UnionType(nonEmptyListOf(a, b, c, b, a).map(AST::TypeReference)).pushRLT()
            unwrap { transformer.transform(type) }.unwrap()
                .shouldBeRight().second shouldBe AST.UnionType(
                nonEmptyListOf(
                    AST.Type("C"),
                    b,
                    a
                ).map(AST::TypeReference)
            )
        }
    }

    afterSpec { clearAllMocks() }
})