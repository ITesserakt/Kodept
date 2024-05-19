package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.RLT

object ObjectLevelGrammar : Grammar<RLT.ObjectLevelNode>() {
    val traitLevel by FunctionGrammar.abstractFunction

    override val rootParser by FunctionGrammar
}