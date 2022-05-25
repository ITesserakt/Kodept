package ru.tesserakt.kodept.parser

import arrow.core.nonEmptyListOf
import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.core.AST

class TypeGrammarTest : WordSpec({
    "simple types" should {
        test(TypeGrammar, "A", AST.Type("A"))
        test(TypeGrammar, "b", null)
        test(TypeGrammar, "A_", AST.Type("A_"))
    }

    "simple tuple types" should {
        test(TypeGrammar, "()", AST.TupleType.unit)
        test(TypeGrammar, "(A)", AST.Type("A"))
        test(TypeGrammar, "(A, B)", AST.TupleType(listOf("A", "B").map(AST::Type)))
    }

    "simple union types" should {
        test(TypeGrammar.union, "()", null)
        test(TypeGrammar.union, "(A)", null)
        test(TypeGrammar, "(A | B)", AST.UnionType(nonEmptyListOf("A", "B").map(AST::Type)))
    }

    "nested types" should {
        test(
            TypeGrammar,
            "(A | (X, Y) | M)",
            AST.UnionType(nonEmptyListOf(AST.Type("A"), AST.TupleType(listOf("X", "Y").map(AST::Type)), AST.Type("M")))
        )

        test(
            TypeGrammar,
            "(A, (C | D), B)",
            AST.TupleType(listOf(AST.Type("A"), AST.UnionType(nonEmptyListOf("C", "D").map(AST::Type)), AST.Type("B")))
        )
        test(
            TypeGrammar,
            "(A, (() | B))",
            AST.TupleType(listOf(AST.Type("A"), AST.UnionType(nonEmptyListOf(AST.TupleType.unit, AST.Type("B")))))
        )
    }
})
