package ru.tesserakt.kodept.lexer

import io.kotest.core.spec.style.StringSpec
import io.kotest.data.forAll
import io.kotest.data.headers
import io.kotest.data.row
import io.kotest.data.table
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.lexer.ExpressionToken.*

class ExpressionTokenTest : StringSpec({
    fun keywordSpecificCases(keyword: ExpressionToken, representation: String) = arrayOf(
        row(keyword, representation, true),
        row(keyword, "${representation}x", false),
        row(keyword, "x$representation", false),
        row(keyword, representation.uppercase(), false),
        row(keyword, "it's_somewhere${representation}_in_word", false)
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
            *keywordSpecificCases(FUN, "fun"),
            *keywordSpecificCases(VAL, "val"),
            *keywordSpecificCases(VAR, "var"),
            *keywordSpecificCases(IF, "if"),
            *keywordSpecificCases(ELIF, "elif"),
            *keywordSpecificCases(ELSE, "else"),
            *keywordSpecificCases(MATCH, "match"),
            *keywordSpecificCases(WHILE, "while"),
            *keywordSpecificCases(MODULE, "module"),
            *keywordSpecificCases(EXTENSION, "extension"),
            row(VAR, "val", false),
        ).forAll(::impliesData)
    }

    "Symbols" {
        table(
            headers("symbol", "input", "matches"),
            *symbolSpecificCases(COMMA, ","),
            *symbolSpecificCases(SEMICOLON, ";"),
            *symbolSpecificCases(LBRACE, "{"),
            *symbolSpecificCases(RBRACE, "}"),
            *symbolSpecificCases(LPAREN, "("),
            *symbolSpecificCases(RPAREN, ")"),
            *symbolSpecificCases(LBRACKET, "["),
            *symbolSpecificCases(RBRACKET, "]"),
            *symbolSpecificCases(TYPE_GAP, "_"),
            *symbolSpecificCases(COLON, ":"),
        ).forAll(::impliesData)
    }

    "Identifiers" {
        table(
            headers("identifier", "input", "matches"),
            row(IDENTIFIER, "test", true),
            row(IDENTIFIER, "test_", true),
            row(IDENTIFIER, "_test", true),
            row(IDENTIFIER, "test132ktn", true),
            row(IDENTIFIER, "test34_1N", true),
            row(IDENTIFIER, "Test", false),
            row(IDENTIFIER, "test irj", false),
            row(IDENTIFIER, "_", false),
            row(IDENTIFIER, "123", false),

            row(TYPE, "Test", true),
            row(TYPE, "Test_", true),
            row(TYPE, "_Test", true),
            row(TYPE, "Test12I_gN", true),
            row(TYPE, "test", false),
            row(TYPE, "Test Int", false),
            row(TYPE, "_", false),
            row(TYPE, "123", false),
        ).forAll(::impliesData)
    }

    "Operators" {
        table(
            headers("operator", "input", "matches"),
            row(PLUS, "+", true),
            row(PLUS_EQUALS, "+=", true),
            row(DIV_EQUALS, "a", false),
            row(AND_LOGIC_EQUALS, "test", false),
            row(GREATER_EQUALS, "123", false)
        ).forAll(::impliesData)
    }

    "Literals" {
        table(
            headers("literal", "input", "matches"),
            row(FLOATING, "0.0", true),
            row(FLOATING, "1,23", false),
            row(FLOATING, ".01", true),
            row(FLOATING, "-12.5", true),
            row(FLOATING, "+.4", true),
            row(FLOATING, "123tst", false),

            row(BINARY, "0b0", true),
            row(BINARY, "0B10", true),
            row(BINARY, "0b1_000", true),
            row(BINARY, "0b1000_", false),
            row(BINARY, "-0b0", false),
            row(BINARY, "_0b0", false),
            row(BINARY, "0b123", false),

            row(OCTAL, "0o0", true),
            row(OCTAL, "0O0", true),
            row(OCTAL, "0o10", true),
            row(OCTAL, "0o1_000", true),
            row(OCTAL, "0o1000_", false),
            row(OCTAL, "-0o0", false),
            row(OCTAL, "_0o0", false),
            row(OCTAL, "0o99", false),

            row(HEX, "0x0", true),
            row(HEX, "0X0", true),
            row(HEX, "0x10", true),
            row(HEX, "0x1_000", true),
            row(HEX, "0x1000_", false),
            row(HEX, "-0x0", false),
            row(HEX, "_0x0", false),
            row(HEX, "0xT", false),

            row(CHAR, "'_'", true),
            row(CHAR, "'_", false),
            row(CHAR, "'ab'", false),
            row(CHAR, """'''""", false),
            row(CHAR, """'\''""", true),
            row(CHAR, """'\\'""", true),

            row(STRING, """"test"""", true),
            row(STRING, """"te\\"st"""", false),
            row(STRING, """"te\\st"""", true),
            row(STRING, """"te\"st"""", true),
            row(STRING, """"test""", false),
        ).forAll(::impliesData)
    }

    "Ignored" {
        table(
            headers("ignore", "input", "matches"),
            row(WHITESPACE, " ", true),
            row(WHITESPACE, "       ", true),
            row(WHITESPACE, "    \n  ", true),
            row(WHITESPACE, "a", false),

            row(NEWLINE, "\n\n", true),
            row(NEWLINE, "a", false),

            row(COMMENT, "// it's a test!", true),
            row(COMMENT, "// test \n is this too?", false),
            row(COMMENT, "a", false),

            row(MULTILINE_COMMENT, "/* it's a test! */", true),
            row(MULTILINE_COMMENT, "/* test \n is this too? */", true),
            row(MULTILINE_COMMENT, "a", false),
        ).forAll(::impliesData)
    }
})
