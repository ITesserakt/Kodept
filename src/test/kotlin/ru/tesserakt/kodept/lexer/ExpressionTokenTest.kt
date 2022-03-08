package ru.tesserakt.kodept.lexer

import io.kotest.core.spec.style.StringSpec
import io.kotest.data.forAll
import io.kotest.data.headers
import io.kotest.data.row
import io.kotest.data.table
import io.kotest.matchers.shouldBe

class ExpressionTokenTest : StringSpec({
    fun keywordSpecificCases(keyword: ExpressionToken, representation: String) = arrayOf(
        row(keyword, representation, true),
        row(keyword, "${representation}x", false),
        row(keyword, "x$representation", false),
        row(keyword, representation.uppercase(), false)
    )

    fun symbolSpecificCases(symbol: ExpressionToken, representation: String) = arrayOf(
        row(symbol, representation, true),
        row(symbol, "${representation}x", false),
        row(symbol, "x$representation", false),
        row(symbol, "test", false),
        row(symbol, "", false)
    )

    fun impliesData(token: ExpressionToken, input: String, matches: Boolean) {
        (token.token.match(input, 0) == input.length && input.isNotEmpty()) shouldBe matches
    }

    "Keywords" {
        table(
            headers("keyword", "input", "matches"),
            *keywordSpecificCases(ExpressionToken.FUN, "fun"),
            *keywordSpecificCases(ExpressionToken.VAL, "val"),
            *keywordSpecificCases(ExpressionToken.VAR, "var"),
            row(ExpressionToken.VAR, "val", false),
        ).forAll(::impliesData)
    }

    "Symbols" {
        table(
            headers("symbol", "input", "matches"),
            *symbolSpecificCases(ExpressionToken.COMMA, ","),
            *symbolSpecificCases(ExpressionToken.SEMICOLON, ";"),
            *symbolSpecificCases(ExpressionToken.LCURVE_BRACKET, "{"),
            *symbolSpecificCases(ExpressionToken.RCURVE_BRACKET, "}"),
            *symbolSpecificCases(ExpressionToken.LPAREN, "("),
            *symbolSpecificCases(ExpressionToken.RPAREN, ")"),
            *symbolSpecificCases(ExpressionToken.LBRACKET, "["),
            *symbolSpecificCases(ExpressionToken.RBRACKET, "]"),
            *symbolSpecificCases(ExpressionToken.TYPE_GAP, "_"),
            *symbolSpecificCases(ExpressionToken.COLON, ":"),
        ).forAll(::impliesData)
    }

    "Identifiers" {
        table(
            headers("identifier", "input", "matches"),
            row(ExpressionToken.IDENTIFIER, "test", true),
            row(ExpressionToken.IDENTIFIER, "test_", true),
            row(ExpressionToken.IDENTIFIER, "_test", true),
            row(ExpressionToken.IDENTIFIER, "test132ktn", true),
            row(ExpressionToken.IDENTIFIER, "test34_1N", true),
            row(ExpressionToken.IDENTIFIER, "Test", false),
            row(ExpressionToken.IDENTIFIER, "test irj", false),
            row(ExpressionToken.IDENTIFIER, "_", false),
            row(ExpressionToken.IDENTIFIER, "123", false),

            row(ExpressionToken.TYPE, "Test", true),
            row(ExpressionToken.TYPE, "Test_", true),
            row(ExpressionToken.TYPE, "_Test", true),
            row(ExpressionToken.TYPE, "Test12I_gN", true),
            row(ExpressionToken.TYPE, "test", false),
            row(ExpressionToken.TYPE, "Test Int", false),
            row(ExpressionToken.TYPE, "_", false),
            row(ExpressionToken.TYPE, "123", false),
        ).forAll(::impliesData)
    }

    "Operators" {
        table(
            headers("operator", "input", "matches"),
            row(ExpressionToken.PLUS, "+", true),
            row(ExpressionToken.PLUS_EQUALS, "+=", true),
            row(ExpressionToken.DIV_EQUALS, "a", false),
            row(ExpressionToken.AND_LOGIC_EQUALS, "test", false),
            row(ExpressionToken.GREATER_EQUALS, "123", false)
        ).forAll(::impliesData)
    }

    "Literals" {
        table(
            headers("literal", "input", "matches"),
            row(ExpressionToken.DECIMAL, "123", true),
            row(ExpressionToken.DECIMAL, "123_000", true),
            row(ExpressionToken.DECIMAL, "1_23", true),
            row(ExpressionToken.DECIMAL, "123_", false),
            row(ExpressionToken.DECIMAL, "-123", true),
            row(ExpressionToken.DECIMAL, "_123", false),
            row(ExpressionToken.DECIMAL, "+123", true),
            row(ExpressionToken.DECIMAL, "test", false),

            row(ExpressionToken.FLOATING, "0.0", true),
            row(ExpressionToken.FLOATING, "1,23", false),
            row(ExpressionToken.FLOATING, ".01", true),
            row(ExpressionToken.FLOATING, "-12.5", true),
            row(ExpressionToken.FLOATING, "+.4", true),
            row(ExpressionToken.FLOATING, "123tst", false),

            row(ExpressionToken.BINARY, "0b0", true),
            row(ExpressionToken.BINARY, "0B10", true),
            row(ExpressionToken.BINARY, "0b1_000", true),
            row(ExpressionToken.BINARY, "0b1000_", false),
            row(ExpressionToken.BINARY, "-0b0", false),
            row(ExpressionToken.BINARY, "_0b0", false),
            row(ExpressionToken.BINARY, "0b123", false),

            row(ExpressionToken.OCTAL, "0o0", true),
            row(ExpressionToken.OCTAL, "0O0", true),
            row(ExpressionToken.OCTAL, "0o10", true),
            row(ExpressionToken.OCTAL, "0o1_000", true),
            row(ExpressionToken.OCTAL, "0o1000_", false),
            row(ExpressionToken.OCTAL, "-0o0", false),
            row(ExpressionToken.OCTAL, "_0o0", false),
            row(ExpressionToken.OCTAL, "0o99", false),

            row(ExpressionToken.HEX, "0x0", true),
            row(ExpressionToken.HEX, "0X0", true),
            row(ExpressionToken.HEX, "0x10", true),
            row(ExpressionToken.HEX, "0x1_000", true),
            row(ExpressionToken.HEX, "0x1000_", false),
            row(ExpressionToken.HEX, "-0x0", false),
            row(ExpressionToken.HEX, "_0x0", false),
            row(ExpressionToken.HEX, "0xT", false),

            row(ExpressionToken.CHAR, "'_'", true),
            row(ExpressionToken.CHAR, "'_", false),
            row(ExpressionToken.CHAR, "'ab'", false),

            row(ExpressionToken.STRING, """"test"""", true),
            row(ExpressionToken.STRING, """"te\\"st"""", true),
            row(ExpressionToken.STRING, """"te\"st"""", false),
            row(ExpressionToken.STRING, """"test""", false),
        ).forAll(::impliesData)
    }

    "Ignored" {
        table(
            headers("ignore", "input", "matches"),
            row(ExpressionToken.WHITESPACE, " ", true),
            row(ExpressionToken.WHITESPACE, "       ", true),
            row(ExpressionToken.WHITESPACE, "    \n  ", true),
            row(ExpressionToken.WHITESPACE, "a", false),

            row(ExpressionToken.NEWLINE, "\n\n", true),
            row(ExpressionToken.NEWLINE, "a", false),

            row(ExpressionToken.COMMENT, "// it's a test!", true),
            row(ExpressionToken.COMMENT, "// test \n is this too?", false),
            row(ExpressionToken.COMMENT, "a", false),

            row(ExpressionToken.MULTILINE_COMMENT, "/* it's a test! */", true),
            row(ExpressionToken.MULTILINE_COMMENT, "/* test \n is this too? */", true),
            row(ExpressionToken.MULTILINE_COMMENT, "a", false),
        ).forAll(::impliesData)
    }
})
