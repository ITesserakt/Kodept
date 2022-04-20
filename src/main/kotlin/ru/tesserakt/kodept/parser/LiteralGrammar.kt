package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.use
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint
import java.math.BigDecimal
import java.math.BigInteger

object LiteralGrammar : Grammar<AST.Literal>() {
    val numberLiteral: Parser<AST.Literal> by (FLOATING use {
        if ('.' in text || 'e' in text)
            AST.FloatingLiteral(BigDecimal(text), toCodePoint())
        else
            AST.DecimalLiteral(BigInteger(text, 10), toCodePoint())
    }) or (BINARY use { AST.BinaryLiteral(BigInteger(text.drop(2), 2), toCodePoint()) }) or
            (OCTAL use { AST.OctalLiteral(BigInteger(text.drop(2), 8), toCodePoint()) }) or
            (HEX use { AST.HexLiteral(BigInteger(text.drop(2), 16), toCodePoint()) })

    val charLiteral by CHAR use { AST.CharLiteral(text.removeSurrounding("'").first(), toCodePoint()) }

    val stringLiteral by STRING use { AST.StringLiteral(text.removeSurrounding("\""), toCodePoint()) }

    override val rootParser by numberLiteral or charLiteral or stringLiteral
}