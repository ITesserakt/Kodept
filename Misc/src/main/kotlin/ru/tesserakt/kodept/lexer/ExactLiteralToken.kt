package ru.tesserakt.kodept.lexer

import com.github.h0tk3y.betterParse.lexer.Token

class ExactLiteralToken(name: String?, val text: String, ignored: Boolean = false) : Token(name, ignored) {
    override fun match(input: CharSequence, fromIndex: Int): Int {
        val whitespaceBefore by lazy { fromIndex == 0 || input.length > fromIndex && !input[fromIndex - 1].isLetterOrDigit() && input[fromIndex - 1] != '_' }
        val match by lazy { input.startsWith(text, fromIndex) }
        val whitespaceAfter by lazy { input.length < fromIndex + text.length + 1 || !input[fromIndex + text.length].isLetterOrDigit() && input[fromIndex + text.length] != '_' }

        return if (whitespaceBefore && match && whitespaceAfter) text.length
        else 0
    }
}

fun exactLiteralToken(name: String?, text: String, ignored: Boolean = false) = ExactLiteralToken(name, text, ignored)
fun exactLiteralToken(text: String, ignored: Boolean = false) = ExactLiteralToken(null, text, ignored)