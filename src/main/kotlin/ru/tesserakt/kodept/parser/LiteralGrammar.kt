package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object LiteralGrammar : Grammar<RLT.Literal>() {
    override val rootParser by (FLOATING map RLT.Literal::Floating) or
            (BINARY or OCTAL or HEX map RLT.Literal::Number) or
            (CHAR or STRING map RLT.Literal::Text)
}