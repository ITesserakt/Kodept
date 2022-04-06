package ru.tesserakt.kodept.lexer

import io.kotest.core.spec.style.StringSpec
import io.kotest.data.forAll
import io.kotest.data.headers
import io.kotest.data.row
import io.kotest.data.table
import io.kotest.matchers.sequences.shouldContainAll
import ru.tesserakt.kodept.lexer.ExpressionToken.*

class LexerTest : StringSpec({
    val lexer = Lexer()

    fun impliesData(input: String, output: Sequence<ExpressionToken>) {
        val tokens = lexer.tokenize(input)
        output.map { it.token } shouldContainAll tokens.map { it.type }.filter { !it.ignored }
    }

    "simple expressions" {
        table(
            headers("input", "tokens"),
            row("val id", sequenceOf(VAL, IDENTIFIER)),
            row("Either[Int, String]", sequenceOf(TYPE, LBRACKET, TYPE, COMMA, TYPE, RBRACKET)),
            row("fun println() {}", sequenceOf(FUN, IDENTIFIER, LPAREN, RPAREN, LBRACE, RBRACE)),
            row("id /= 12", sequenceOf(IDENTIFIER, DIV_EQUALS, FLOATING)),
            row("Int // -2^32..2^32 - 1", sequenceOf(TYPE)),
            row(
                "F[_]: Functor[_]",
                sequenceOf(TYPE, LBRACKET, TYPE_GAP, RBRACKET, COLON, TYPE, LBRACKET, TYPE_GAP, RBRACKET)
            ),
            row("1 < 2 : List[Double]", sequenceOf(FLOATING, LESS, FLOATING, COLON, TYPE, LBRACKET, TYPE, RBRACKET))
        ).forAll(::impliesData)
    }

    "complex expressions" {
        table(
            headers("input", "output"),
            row(
                "output.map { it.token } shouldContainAll tokens.map { it.type }.filter { !it.ignored }",
                sequenceOf(
                    IDENTIFIER,
                    DOT,
                    IDENTIFIER,
                    LBRACE,
                    IDENTIFIER,
                    DOT,
                    IDENTIFIER,
                    RBRACE,
                    IDENTIFIER,
                    IDENTIFIER,
                    DOT,
                    IDENTIFIER,
                    LBRACE,
                    IDENTIFIER,
                    DOT,
                    IDENTIFIER,
                    RBRACE,
                    DOT,
                    IDENTIFIER,
                    LBRACE,
                    NOT_LOGIC,
                    IDENTIFIER,
                    DOT,
                    IDENTIFIER,
                    RBRACE
                )
            ),
            row(
                """tasks.withType[KotlinCompile] {
                    kotlinOptions.jvmTarget = "16"
                }""".trimIndent(),
                sequenceOf(
                    IDENTIFIER,
                    DOT,
                    IDENTIFIER,
                    LBRACKET,
                    TYPE,
                    RBRACKET,
                    LBRACE,
                    IDENTIFIER,
                    DOT,
                    IDENTIFIER,
                    EQUALS,
                    STRING,
                    RBRACE
                )
            )
        ).forAll(::impliesData)
    }
})
