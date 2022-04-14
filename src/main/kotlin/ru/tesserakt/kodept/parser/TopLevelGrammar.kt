package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint

object TopLevelGrammar : Grammar<AST.TopLevelDecl>() {
    private val functionStatement by FunctionGrammar

    val enumStatement by ENUM * (STRUCT or CLASS) * TYPE * -LBRACE * trailing(
        TYPE, COMMA, atLeast = 1
    ) * -RBRACE map { (enumToken, modifier, name, entries) ->
        AST.EnumDecl(name.text,
            modifier.type == STRUCT.token,
            entries.map { AST.EnumDecl.Entry(it.text, it.toCodePoint()) },
            enumToken.toCodePoint())
    }

    val traitStatement by TRAIT * TYPE * -LBRACE * trailing(ObjectLevelGrammar) * -RBRACE map { (traitToken, name, rest) ->
        AST.TraitDecl(name.text, rest, traitToken.toCodePoint())
    }

    val structStatement by STRUCT * TYPE * optional(
        -LPAREN * trailing(IDENTIFIER * TypeGrammar.strict, COMMA) * -RPAREN
    ) * optional(
        -LBRACE * trailing(ObjectLevelGrammar) * -RBRACE
    ) map { (structToken, name, allocated, rest) ->
        AST.StructDecl(
            name.text,
            allocated.orEmpty().map { AST.StructDecl.Parameter(it.t1.text, it.t2, it.t1.toCodePoint()) },
            rest.orEmpty(),
            structToken.toCodePoint()
        )
    }

    override val rootParser: Parser<AST.TopLevelDecl> by
    structStatement or traitStatement or enumStatement or functionStatement
}