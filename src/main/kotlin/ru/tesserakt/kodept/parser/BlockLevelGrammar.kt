package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.combinators.unaryMinus
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.parser.AST.ExpressionList

object BlockLevelGrammar : Grammar<AST.BlockLevelDecl>() {
    private val expression by OperatorGrammar
    private val functionStatement by FunctionGrammar

    val bracedDecls = -LBRACE * trailing(this) * -RBRACE map ::ExpressionList
    val simple = -FLOW * OperatorGrammar
    val body = simple or bracedDecls

    val varDecl by (VAL or VAR) * TermGrammar.variableReference * TypeGrammar.optional map {
        AST.VariableDecl(it.t2.name, it.t1.type == VAR.token, it.t3)
    }

    private fun expandAssignment(a: AST.Expression, op: TokenMatch, b: AST.Expression) = when (op.type) {
        PLUS_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Add)
        SUB_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Sub)
        TIMES_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Mul)
        DIV_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Div)
        MOD_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Mod)
        POW_EQUALS.token -> AST.Mathematical(a, b, AST.Mathematical.Kind.Pow)
        OR_LOGIC_EQUALS.token -> AST.Logical(a, b, AST.Logical.Kind.Disjunction)
        AND_LOGIC_EQUALS.token -> AST.Logical(a, b, AST.Logical.Kind.Conjunction)
        OR_BIT_EQUALS.token -> AST.Binary(a, b, AST.Binary.Kind.Or)
        AND_BIT_EQUALS.token -> AST.Binary(a, b, AST.Binary.Kind.And)
        XOR_BIT_EQUALS.token -> AST.Binary(a, b, AST.Binary.Kind.Xor)
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
        AST.Assignment(decl, expandAssignment(decl, op, expr))
    }

    override val rootParser by functionStatement or initialization or varDecl or assignment or expression
}