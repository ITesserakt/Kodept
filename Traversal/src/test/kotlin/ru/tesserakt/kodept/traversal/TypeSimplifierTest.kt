package ru.tesserakt.kodept.traversal

import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import io.mockk.clearAllMocks
import io.mockk.every
import io.mockk.mockk
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.CodePoint
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.RLT

class TypeSimplifierTest : StringSpec({
    lateinit var tupleRLT: RLT.TupleType
    lateinit var unionRLT: RLT.UnionType

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
            val type = AST.TupleType(listOf("A", "B").map(AST::Type))
            unwrap { transformer.transform(type) }.toEither().shouldBeRight(type)
        }

//        "Proper union types should not change" {
//            val type = AST.UnionType(nonEmptyListOf("A", "B").map(AST::Type)).apply { _rlt = unionRLT }
//            unwrap { transformer.transform(type) }.toEither().shouldBeRight(type)
//        }
//
//        "Single type in tuple should be aligned with it" {
//            val inner = AST.TupleType(listOf("A", "B").map(AST::Type))
//            val type = AST.TupleType(listOf(inner)).apply { _rlt = tupleRLT }
//            unwrap { transformer.transform(type) }.unwrap().shouldBeRight().second shouldBe inner
//        }
//
//        "Single union types should cause crash" {
//            val union = AST.UnionType(nonEmptyListOf("A").map(AST::Type)).apply { _rlt = unionRLT }
//            unwrap { transformer.transform(union) }.toEither()
//                .shouldBeLeft().head.severity shouldBe Report.Severity.CRASH
//        }
//
//        "Identical items in union should be aligned and flattened" {
//            val a = AST.Type("A")
//            val b = AST.TupleType(listOf("B", "C").map(AST::Type)).apply { _rlt = tupleRLT }
//            val c = AST.UnionType(nonEmptyListOf(AST.Type("C"), a)).apply { _rlt = unionRLT }
//            val type = AST.UnionType(nonEmptyListOf(a, b, c, b, a)).apply { _rlt = unionRLT }
//            unwrap { transformer.transform(type) }.unwrap()
//                .shouldBeRight().second shouldBe AST.UnionType(nonEmptyListOf(AST.Type("C"), b, a))
//        }
    }

    afterSpec { clearAllMocks() }
})