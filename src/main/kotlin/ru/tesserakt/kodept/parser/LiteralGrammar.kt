package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.use
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import java.math.BigDecimal
import java.math.BigInteger

object LiteralGrammar : Grammar<AST.Literal>() {
    val numberLiteral by (FLOATING use {
        if ('.' in text || 'e' in text)
            AST.FloatingLiteral(BigDecimal(text))
        else
            AST.DecimalLiteral(BigInteger(text, 10))
    }) or (BINARY use { AST.BinaryLiteral(BigInteger(text.drop(2), 2)) }) or
            (OCTAL use { AST.OctalLiteral(BigInteger(text.drop(2), 8)) }) or
            (HEX use { AST.HexLiteral(BigInteger(text.drop(2), 16)) })

    val charLiteral by CHAR use { AST.CharLiteral(text.removeSurrounding("'").first()) }

    val stringLiteral by STRING use { AST.StringLiteral(text.removeSurrounding("\"")) }

    override val rootParser by numberLiteral or charLiteral or stringLiteral
}