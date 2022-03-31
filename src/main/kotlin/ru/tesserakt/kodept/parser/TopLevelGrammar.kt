package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object TopLevelGrammar : Grammar<AST.TopLevelDecl>() {
    private val functionStatement by FunctionGrammar

    val enumStatement by -ENUM * (STRUCT or CLASS) * TYPE * -LBRACE * trailing(
        TYPE, COMMA, atLeast = 1
    ) * -RBRACE map { (modifier, name, entries) ->
        AST.EnumDecl(name.text, modifier.type == STRUCT.token, entries.map { AST.EnumDecl.Entry(it.text) })
    }

    val traitStatement by -TRAIT * TYPE * -LBRACE * trailing(ObjectLevelGrammar) * -RBRACE map { (name, rest) ->
        AST.TraitDecl(name.text, rest)
    }

    val structStatement by -STRUCT * TYPE * optional(
        -LPAREN * trailing(IDENTIFIER * TypeGrammar.strict, COMMA) * -RPAREN
    ) * optional(
        -LBRACE * trailing(ObjectLevelGrammar) * -RBRACE
    ) map { (name, allocated, rest) ->
        AST.StructDecl(
            name.text,
            allocated.orEmpty().map { AST.StructDecl.Parameter(it.t1.text, it.t2) },
            rest.orEmpty()
        )
    }

    override val rootParser: Parser<AST.TopLevelDecl> by
    structStatement or traitStatement or enumStatement or functionStatement
}