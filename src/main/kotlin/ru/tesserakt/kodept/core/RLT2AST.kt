package ru.tesserakt.kodept.core

import arrow.core.partially2
import arrow.core.partially3
import com.github.h0tk3y.betterParse.lexer.Token
import org.jetbrains.annotations.TestOnly
import ru.tesserakt.kodept.lexer.ExpressionToken
import ru.tesserakt.kodept.parser.RLT

private fun expandCompound(left: AST.Lvalue, right: AST.Expression, token: Token) = when (token) {
    ExpressionToken.PLUS_EQUALS.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Add)
    ExpressionToken.SUB_EQUALS.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Sub)
    ExpressionToken.TIMES_EQUALS.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Mul)
    ExpressionToken.DIV_EQUALS.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Div)
    ExpressionToken.MOD_EQUALS.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Mod)
    ExpressionToken.POW_EQUALS.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Pow)
    ExpressionToken.OR_LOGIC_EQUALS.token -> AST::Logical.partially3(AST.Logical.Kind.Disjunction)
    ExpressionToken.AND_LOGIC_EQUALS.token -> AST::Logical.partially3(AST.Logical.Kind.Conjunction)
    ExpressionToken.OR_BIT_EQUALS.token -> AST::Binary.partially3(AST.Binary.Kind.Or)
    ExpressionToken.AND_BIT_EQUALS.token -> AST::Binary.partially3(AST.Binary.Kind.And)
    ExpressionToken.XOR_BIT_EQUALS.token -> AST::Binary.partially3(AST.Binary.Kind.Xor)
    ExpressionToken.EQUALS.token -> { _, r -> r }
    else -> throw IllegalStateException("Impossible")
}.let {
    fun copyLvalue(value: AST.Lvalue) = when (value) {
        is AST.Dereference -> value.copy()
        is AST.FunctionCall -> value.copy()
        is AST.Reference -> value.copy()
        is AST.TypeReference -> value.copy()
    }

    AST.Assignment(left, it(copyLvalue(left), right))
}

context (RLT.BinaryOperation)
        private fun expandBinary(left: AST.Expression, right: AST.Expression, token: Token) = when (token) {
    ExpressionToken.DOT.token -> AST::Dereference
    ExpressionToken.PLUS.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Add)
    ExpressionToken.SUB.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Sub)
    ExpressionToken.TIMES.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Mul)
    ExpressionToken.DIV.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Div)
    ExpressionToken.MOD.token -> AST::Mathematical.partially3(AST.Mathematical.Kind.Mod)
    ExpressionToken.OR_BIT.token -> AST::Binary.partially3(AST.Binary.Kind.Or)
    ExpressionToken.AND_BIT.token -> AST::Binary.partially3(AST.Binary.Kind.And)
    ExpressionToken.XOR_BIT.token -> AST::Binary.partially3(AST.Binary.Kind.Xor)
    ExpressionToken.OR_LOGIC.token -> AST::Logical.partially3(AST.Logical.Kind.Disjunction)
    ExpressionToken.AND_LOGIC.token -> AST::Logical.partially3(AST.Logical.Kind.Conjunction)
    ExpressionToken.LESS.token -> AST::Comparison.partially3(AST.Comparison.Kind.Less)
    ExpressionToken.LESS_EQUALS.token -> AST::Comparison.partially3(AST.Comparison.Kind.LessEqual)
    ExpressionToken.EQUIV.token -> AST::Comparison.partially3(AST.Comparison.Kind.Equal)
    ExpressionToken.NOT_EQUIV.token -> AST::Comparison.partially3(AST.Comparison.Kind.NonEqual)
    ExpressionToken.GREATER_EQUALS.token -> AST::Comparison.partially3(AST.Comparison.Kind.GreaterEqual)
    ExpressionToken.GREATER.token -> AST::Comparison.partially3(AST.Comparison.Kind.Greater)
    ExpressionToken.SPACESHIP.token -> AST::Comparison.partially3(AST.Comparison.Kind.Complex)
    ExpressionToken.ELVIS.token -> AST::Elvis
    else -> throw IllegalStateException("Impossible")
}(left, right).withRLT()

private fun expandUnary(expression: AST.Expression, token: Token) = when (token) {
    ExpressionToken.PLUS.token -> AST.Absolution(expression)
    ExpressionToken.SUB.token -> AST.Negation(expression)
    ExpressionToken.NOT_BIT.token -> AST.BitInversion(expression)
    ExpressionToken.NOT_LOGIC.token -> AST.Inversion(expression)
    else -> throw IllegalStateException("Impossible")
}

private fun RLT.Context.convert(): AST.ResolutionContext = when (this) {
    is RLT.Context.Global -> AST.ResolutionContext(true, emptyList())
    is RLT.Context.Inner -> AST.ResolutionContext(global, parent.convert().chain + AST.Type(type.ref.text.value()))
    RLT.Context.Local -> AST.ResolutionContext(false, emptyList())
}

private fun RLT.UserSymbol.Type.convert() = AST.Type(text.value()).withRLT()

private fun RLT.Variable.convert() = when (this) {
    is RLT.Variable.Immutable -> AST::VariableDecl.partially2(false)
    is RLT.Variable.Mutable -> AST::VariableDecl.partially2(true)
}.invoke(with(RLT.Reference(id)) { AST.Reference(id.text.value()).withRLT() }, type?.convert()).withRLT()

private fun RLT.Literal.convert(): AST.Literal = when (this) {
    is RLT.Literal.Floating -> when {
        '.' in text.value() || text.value().contains('e', true) -> AST.FloatingLiteral(text.value().toBigDecimal())
        else -> AST.DecimalLiteral(text.value().toBigInteger())
    }.withRLT()

    is RLT.Literal.Number -> when (text.value()[1].lowercaseChar()) {
        'o' -> AST::OctalLiteral to 8
        'b' -> AST::BinaryLiteral to 2
        'x' -> AST::HexLiteral to 16
        else -> throw IllegalStateException("Impossible")
    }.let { it.first(text.value().drop(2).toBigInteger(it.second)) }.withRLT()

    is RLT.Literal.Text -> when (text.value().first()) {
        '\'' -> AST.CharLiteral(text.value().removeSurrounding("'").first())
        '"' -> AST.StringLiteral(text.value().removeSurrounding("\""))
        else -> throw IllegalStateException("Impossible")
    }.withRLT()

    is RLT.Literal.Tuple -> AST.TupleLiteral(expressions.map(RLT.ExpressionNode::convert)).withRLT()
}

private fun RLT.Module.convert(): AST.ModuleDecl = when (this) {
    is RLT.Module.Global -> AST::ModuleDecl.partially2(true)
    is RLT.Module.Ordinary -> AST::ModuleDecl.partially2(false)
}.invoke(id.text.value(), rest.map(RLT.TopLevelNode::convert).toMutableList()).withRLT()

private fun RLT.ParameterTuple.convert(): AST.TupleLiteral =
    AST.TupleLiteral(params.map(RLT.Parameter::convert)).withRLT()

private fun RLT.MaybeTypedParameter.convert() =
    AST.InferredParameter(id.text.value(), type?.convert()).withRLT()

private fun RLT.MaybeTypedParameterTuple.convert() = params.map { it.convert() }

private fun RLT.TypedParameter.convert() = AST.Parameter(id.text.value(), type.convert()).withRLT()

private fun RLT.TypedParameterTuple.convert() = params.map { it.convert() }

private fun RLT.If.Else.convert() = AST.IfExpr.ElseExpr(body.convert()).withRLT()

private fun RLT.If.Elif.convert() =
    AST.IfExpr.ElifExpr(condition.convert(), body.convert()).withRLT()

private fun RLT.Function.Bodied.convert() = AST.FunctionDecl(
    id.text.value(),
    params.flatMap { it.convert() },
    returnType?.convert(),
    body.convert()
).withRLT()

private fun RLT.Enum.convert() = when (this) {
    is RLT.Enum.Heap -> AST::EnumDecl.partially2(false)
    is RLT.Enum.Stack -> AST::EnumDecl.partially2(true)
}.invoke(id.text.value(), rest.map {
    with(it) { AST.EnumDecl.Entry(it.text.value()).withRLT() }
}.toMutableList()).withRLT()

private fun RLT.TopLevelNode.convert(): AST.TopLevel = when (this) {
    is RLT.Function.Bodied -> convert()

    is RLT.Enum -> convert()

    is RLT.Struct -> AST.StructDecl(id.text.value(),
        varsToAlloc.map { it.convert() },
        rest.map { it.convert() }).withRLT()

    is RLT.Trait -> AST.TraitDecl(id.text.value(), rest.map { it.convert() }).withRLT()
}

private fun RLT.StructLevelNode.convert() = when (this) {
    is RLT.Function.Bodied -> convert()
}

private fun RLT.ObjectLevelNode.convert(): AST.TraitLevel = when (this) {
    is RLT.Function.Bodied -> convert()

    is RLT.Function.Abstract -> AST.AbstractFunctionDecl(
        id.text.value(),
        params.flatMap { it.convert() },
        returnType?.convert()
    ).withRLT()
}

private fun RLT.BlockLevelNode.convert(): AST.BlockLevel = when (this) {
    is RLT.StatementNode -> convert()
    is RLT.ExpressionNode -> convert()
}

private fun RLT.StatementNode.convert(): AST.BlockLevel = when (this) {
    is RLT.Assignment -> when (this) {
        is RLT.InitializedAssignment -> AST.InitializedVar(lvalue.convert(), expression.convert())
        else -> expandCompound(
            lvalue.convert(),
            expression.convert(),
            equals.tokenType
        )
    }.withRLT()

    is RLT.Function.Bodied -> convert()

    is RLT.CompoundAssignment -> expandCompound(
        lvalue.convert(),
        expression.convert(),
        compoundOperator.tokenType
    ).withRLT()

    is RLT.Variable -> convert()
}

private fun RLT.Body.Block.convert(): AST.Expression = when (block.size) {
    0 -> AST.TupleLiteral.unit.withRLT()
    1 -> {
        val head = block.first()
        if (head is RLT.ExpressionNode)
            head.convert()
        else
            AST.ExpressionList(listOf(head.convert(), AST.TupleLiteral.unit.withRLT())).withRLT()
    }

    else -> AST.ExpressionList(block.map { it.convert() }).withRLT()
}

private fun RLT.ExpressionNode.convert(): AST.Expression = when (this) {
    is RLT.BinaryOperation -> expandBinary(left.convert(), right.convert(), op.tokenType)

    is RLT.Body.Block -> convert()

    is RLT.Body.Expression -> expression.convert()

    is RLT.If -> AST.IfExpr(
        condition.convert(),
        body.convert(),
        elifs.map { it.convert() },
        el?.convert()
    ).withRLT()

    is RLT.UnaryOperation -> expandUnary(expression.convert(), op.tokenType).withRLT()

    is RLT.While -> AST.WhileExpr(condition.convert(), body.convert()).withRLT()

    is RLT.Parameter -> id.convert()

    is RLT.TermNode -> convert()

    is RLT.Literal -> convert()
    is RLT.ParameterTuple -> convert()
}

private fun RLT.Lvalue.convert(): AST.Lvalue = when (this) {
    is RLT.Variable -> convert().reference
    is RLT.TermNode -> convert()
}

private fun RLT.TermNode.convert(): AST.Lvalue = when (this) {
    is RLT.Application -> AST.FunctionCall(expr.convert(),
        params.map(RLT.ParameterTuple::convert).filterNot { it.items.isEmpty() }).withRLT()

    is RLT.ContextualReference -> when (ref) {
        is RLT.UserSymbol.Identifier -> AST.Reference(ref.text.value(), context.convert()).withRLT()
        is RLT.UserSymbol.Type -> AST.TypeReference(ref.convert(), context.convert()).withRLT()
    }

    is RLT.Reference -> when (ref) {
        is RLT.UserSymbol.Identifier -> AST.Reference(ref.text.value()).withRLT()
        is RLT.UserSymbol.Type -> AST.TypeReference(ref.convert()).withRLT()
    }
}

fun RLT.TypeNode.convert(): AST.TypeExpression = when (this) {
    is RLT.TupleType -> AST.TupleType(types.map { it.convert() }).withRLT()
    is RLT.UserSymbol.Type -> convert()
    is RLT.UnionType -> AST.UnionType(types.map { it.convert() }).withRLT()
}

fun RLT.File.convert(): AST.FileDecl = AST.FileDecl(moduleList.map(RLT.Module::convert)).withRLT()

@TestOnly
fun RLT.Node.convert() = when (this) {
    is RLT.TopLevelNode -> convert()
    is RLT.ObjectLevelNode -> convert()
    is RLT.BlockLevelNode -> convert()
    is RLT.TypeNode -> convert()
    is RLT.If.Elif -> convert()
    is RLT.If.Else -> convert()
    is RLT.File -> convert()
    is RLT.Module -> convert()
    is RLT.MaybeTypedParameter -> convert()
    is RLT.ParameterTuple -> convert()
    is RLT.UserSymbol.Identifier -> throw IllegalStateException("Thrown out")
    is RLT.Keyword -> throw IllegalStateException("Thrown out")
    is RLT.Symbol -> throw IllegalStateException("Thrown out")
    is RLT.MaybeTypedParameterTuple -> throw IllegalStateException("Thrown out")
    is RLT.TypedParameter -> convert()
    is RLT.InitializedAssignment -> convert()
}