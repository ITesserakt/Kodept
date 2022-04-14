package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint
import ru.tesserakt.kodept.parser.AST.TypeExpression

object TypeGrammar : Grammar<TypeExpression>() {
    val optional by optional(-COLON * this)
    val strict by -COLON * TYPE use { TypeExpression(text, toCodePoint()) }

    override val rootParser by (TYPE or TYPE_GAP) use { TypeExpression(text, toCodePoint()) }
}