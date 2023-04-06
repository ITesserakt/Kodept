package ru.tesserakt.kodept.traversal

import arrow.core.nonEmptyListOf
import io.kotest.assertions.arrow.core.shouldBeLeft
import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.types.shouldBeTypeOf
import io.mockk.clearAllMocks
import io.mockk.every
import io.mockk.mockk
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.CodePoint
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.error.Report

class TypeSimplifierTest : StringSpec({
    val tupleRLT: RLT.TupleType = mockk {
        every { position } returns CodePoint(0, 0)
    }
    val unionRLT: RLT.UnionType = mockk {
        every { position } returns CodePoint(0, 0)
    }
    val typeRLT: RLT.UserSymbol.Type = mockk {
        every { position } returns CodePoint(0, 0)
    }

    fun AST.UnionType.pushRLT() = with(unionRLT) { withRLT() }
    fun AST.TupleType.pushRLT() = with(tupleRLT) { withRLT() }
    fun AST.Type.pushRLT() = with(typeRLT) { withRLT() }

    with(Filepath("TEST FILE")) {
        val transformer = TypeSimplifier

        "simple type should not change" {
            val type = AST.Type("A").pushRLT()
            unwrap { transformer.transform(type)() }.toEither().shouldBeRight() shouldBe type
        }

        "Compound tuple types should not change" {
            val type = AST.TupleType(listOf("A", "B").map(AST::Type).map(AST::TypeReference)).pushRLT()
            unwrap { transformer.transform(type)() }.toEither().shouldBeRight() shouldBe type
        }

        "Proper union types should not change" {
            val type = AST.UnionType(nonEmptyListOf("A", "B").map(AST::Type).map(AST::TypeReference)).pushRLT()
            unwrap { transformer.transform(type)() }.toEither().shouldBeRight()
                .shouldBeTypeOf<AST.UnionType>().items shouldBe type.items
        }

        "Single type in tuple should be aligned with it" {
            val inner = AST.TupleType(listOf("A", "B").map(AST::Type).map(AST::TypeReference)).pushRLT()
            val type = AST.TupleType(listOf(inner)).pushRLT()
            unwrap { transformer.transform(type)() }.unwrap().shouldBeRight().second shouldBe inner
        }

        "Single union types should cause crash" {
            val union = AST.UnionType(nonEmptyListOf("A").map(AST::Type).map(AST::TypeReference)).pushRLT()
            unwrap { transformer.transform(union)() }.toEither()
                .shouldBeLeft().head.severity shouldBe Report.Severity.CRASH
        }

        "Identical items in union should be aligned and flattened" {
            val a = AST.Type("A").let(AST::TypeReference)
            val b = AST.TupleType(listOf("B", "C").map(AST::Type).map(AST::TypeReference)).pushRLT()
            val c = AST.UnionType(nonEmptyListOf(AST.Type("C").let(AST::TypeReference), a)).pushRLT()
            val type = AST.UnionType(nonEmptyListOf(a, b, c, b, a)).pushRLT()
            unwrap { transformer.transform(type)() }.unwrap()
                .shouldBeRight().second.toString() shouldBe
                    AST.UnionType(nonEmptyListOf(AST.Type("C").let(AST::TypeReference), a, b)).toString()
        }
    }

    afterSpec { clearAllMocks() }
})