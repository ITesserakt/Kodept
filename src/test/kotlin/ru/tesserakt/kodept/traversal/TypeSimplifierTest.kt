package ru.tesserakt.kodept.traversal

import arrow.core.nonEmptyListOf
import io.kotest.assertions.arrow.core.shouldBeLeft
import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.mockk.clearAllMocks
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.rlt
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.lexer.CodePoint
import ru.tesserakt.kodept.parser.RLT

class TypeSimplifierTest : StringSpec({
    beforeSpec {
        val tupleRLT = mockk<RLT.TupleType> {
            every { position } returns CodePoint(0, 0)
        }
        val unionRLT = mockk<RLT.UnionType> {
            every { position } returns CodePoint(0, 0)
        }
        mockkStatic(AST.TupleType::rlt)
        mockkStatic(AST.UnionType::rlt)
        every { any<AST.TupleType>().rlt } returns tupleRLT
        every { any<AST.UnionType>().rlt } returns unionRLT
    }

    with("TEST FILE" as Filename) {
        val transformer = TypeSimplifier

        "simple type should not change" {
            val type = AST.Type("A")
            unwrap { transformer.transform(type) }.toEither().shouldBeRight(type)
        }

        "Compound tuple types should not change" {
            val type = AST.TupleType(listOf("A", "B").map(AST::Type))
            unwrap { transformer.transform(type) }.toEither().shouldBeRight(type)
        }

        "Proper union types should not change" {
            val type = AST.UnionType(nonEmptyListOf("A", "B").map(AST::Type))
            unwrap { transformer.transform(type) }.toEither().shouldBeRight(type)
        }

        "Single type in tuple should be aligned with it" {
            val inner = AST.TupleType(listOf("A", "B").map(AST::Type))
            val type = AST.TupleType(listOf(inner))
            unwrap { transformer.transform(type) }.unwrap().shouldBeRight().second shouldBe inner
        }

        "Single union types should cause crash" {
            val union = AST.UnionType(nonEmptyListOf("A").map(AST::Type))
            unwrap { transformer.transform(union) }.toEither()
                .shouldBeLeft().head.severity shouldBe Report.Severity.CRASH
        }

        "Identical items in union should be aligned and flattened" {
            val a = AST.Type("A")
            val b = AST.TupleType(listOf("B", "C").map(AST::Type))
            val c = AST.UnionType(nonEmptyListOf(AST.Type("C"), a))
            val type = AST.UnionType(nonEmptyListOf(a, b, c, b, a))
            unwrap { transformer.transform(type) }.unwrap()
                .shouldBeRight().second shouldBe AST.UnionType(nonEmptyListOf(AST.Type("C"), b, a))
        }
    }

    afterSpec { clearAllMocks() }
})