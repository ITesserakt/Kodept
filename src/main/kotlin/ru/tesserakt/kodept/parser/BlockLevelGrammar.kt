package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.combinators.unaryMinus
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.lexer.CodePoint
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint

object BlockLevelGrammar : Grammar<AST.BlockLevelDecl>() {
    private val expression by OperatorGrammar
    private val functionStatement by FunctionGrammar

    val bracedDecls = LBRACE * trailing(this) * -RBRACE map {
        AST.ExpressionList(it.t2, it.t1.toCodePoint())
    }
    val simple = -FLOW * OperatorGrammar
    val body = simple or bracedDecls

    val varDecl by (VAL or VAR) * TermGrammar.variableReference * TypeGrammar.optional map {
        AST.VariableDecl(it.t2.name, it.t1.type == VAR.token, it.t3, it.t1.toCodePoint())
    }

    private fun expandAssignment(a: AST.Expression, op: TokenMatch, b: AST.Expression, where: CodePoint) =
        when (op.type) {
            PLUS_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Add, where)
            SUB_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Sub, where)
            TIMES_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Mul, where)
            DIV_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Div, where)
            MOD_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Mod, where)
            POW_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Pow, where)
            OR_LOGIC_EQUALS.token -> AST.Logical(a, b, AST.Logical.Kind.Disjunction, where)
            AND_LOGIC_EQUALS.token -> AST.Logical(a, b, AST.Logical.Kind.Conjunction, where)
            OR_BIT_EQUALS.token -> AST.Binary(a, b, AST.Binary.Kind.Or, where)
            AND_BIT_EQUALS.token -> AST.Binary(a, b, AST.Binary.Kind.And, where)
            XOR_BIT_EQUALS.token -> AST.Binary(a, b, AST.Binary.Kind.Xor, where)
            EQUALS.token -> b
            else -> throw IllegalArgumentException("Impossible")
        }

    val initialization by varDecl * EQUALS * (bracedDecls or OperatorGrammar) map { (decl, _, expr) ->
        AST.InitializedVar(decl, expr)
    }

    val assignment by TermGrammar *
            (PLUS_EQUALS or SUB_EQUALS or TIMES_EQUALS or DIV_EQUALS or MOD_EQUALS or POW_EQUALS or
                    OR_LOGIC_EQUALS or AND_LOGIC_EQUALS or
                    OR_BIT_EQUALS or AND_BIT_EQUALS or XOR_BIT_EQUALS or
                    EQUALS) * (bracedDecls or OperatorGrammar) map { (decl, op, expr) ->
        AST.Assignment(decl, expandAssignment(decl, op, expr, op.toCodePoint()), op.toCodePoint())
    }

    override val rootParser by functionStatement or initialization or varDecl or assignment or expression or bracedDecls
}