package ru.tesserakt.kodept.parser

import arrow.core.nonEmptyListOf
import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.core.AST

class TypeGrammarTest : WordSpec({
    "simple types" should {
        test(TypeGrammar, "A", AST.Type("A").let(AST::TypeReference))
        test(TypeGrammar, "b", null)
        test(TypeGrammar, "A_", AST.Type("A_").let(AST::TypeReference))
    }

    "simple tuple types" should {
        test(TypeGrammar, "()", AST.TupleType.unit.let(AST::TypeReference))
        test(
            TypeGrammar,
            "(A)",
            AST.TupleType(listOf("A").map(AST::Type).map(AST::TypeReference)).let(AST::TypeReference)
        )
        test(
            TypeGrammar,
            "(A, B)",
            AST.TupleType(listOf("A", "B").map(AST::Type).map(AST::TypeReference)).let(AST::TypeReference)
        )
    }

    "simple union types" should {
        test(TypeGrammar.union, "()", null)
        test(TypeGrammar.union, "(A)", null)
        test(
            TypeGrammar,
            "(A | B)",
            AST.UnionType(nonEmptyListOf("A", "B").map(AST::Type).map(AST::TypeReference)).let(AST::TypeReference)
        )
    }

    "nested types" should {
        test(
            TypeGrammar,
            "(A | (X, Y) | M)",
            AST.UnionType(
                nonEmptyListOf(
                    AST.Type("A"),
                    AST.TupleType(listOf("X", "Y").map(AST::Type).map(AST::TypeReference)),
                    AST.Type("M")
                ).map(AST::TypeReference)
            ).let(AST::TypeReference)
        )

        test(
            TypeGrammar,
            "(A, (C | D), B)",
            AST.TupleType(
                listOf(
                    AST.Type("A"),
                    AST.UnionType(nonEmptyListOf("C", "D").map(AST::Type).map(AST::TypeReference)),
                    AST.Type("B")
                ).map(AST::TypeReference)
            ).let(AST::TypeReference)
        )
        test(
            TypeGrammar,
            "(A, (() | B))",
            AST.TupleType(
                listOf(
                    AST.Type("A"),
                    AST.UnionType(
                        nonEmptyListOf(
                            AST.TupleType.unit,
                            AST.Type("B")
                        ).map(AST::TypeReference)
                    )
                ).map(AST::TypeReference)
            ).let(AST::TypeReference)
        )
    }
})
