package ru.tesserakt.kodept.parser

import io.kotest.core.spec.style.WordSpec

class LiteralGrammarTest : WordSpec({
    val grammar = LiteralGrammar

    "number literals" should {
        test(grammar.numberLiteral, "123", AST.DecimalLiteral(123.toBigInteger()))
        test(grammar.numberLiteral, "0x14", AST.HexLiteral(20.toBigInteger()))
        test(grammar.numberLiteral, "0b012", null)
        test(grammar.numberLiteral, "0b1010", AST.BinaryLiteral(10.toBigInteger()))
        test(grammar.numberLiteral, "0.001", AST.FloatingLiteral(0.001.toBigDecimal()))
        test(grammar.numberLiteral, "1.0e7", AST.FloatingLiteral(1e7.toBigDecimal()))
    }

    "literals" should {
        test(grammar, "'a'", AST.CharLiteral('a'))
        test(grammar, """"test"""", AST.StringLiteral("test"))
        test(grammar, "0x256", AST.HexLiteral(598.toBigInteger()))
        test(grammar, "'ab'", null)
    }
})