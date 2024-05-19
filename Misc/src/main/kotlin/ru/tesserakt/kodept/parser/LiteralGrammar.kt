package ru.tesserakt.kodept.parser

import arrow.core.Eval
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.use
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.symbol
import ru.tesserakt.kodept.core.toCodePoint
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object LiteralGrammar : Grammar<RLT.Literal>() {
    val floating by FLOATING use { RLT.Literal.Floating(Eval.later { text }, toCodePoint()) }
    val number by BINARY or OCTAL or HEX use { RLT.Literal.Number(Eval.later { text }, toCodePoint()) }
    val char by CHAR use { RLT.Literal.Text(Eval.later { text }, toCodePoint()) }
    val string by STRING use { RLT.Literal.Text(Eval.later { text }, toCodePoint()) }
    val tuple by LPAREN and strictTrailing(OperatorGrammar, COMMA) * RPAREN map {
        RLT.Literal.Tuple(
            it.t1.symbol(),
            it.t2,
            it.t3.symbol()
        )
    }

    override val rootParser by floating or number or char or string or tuple
}