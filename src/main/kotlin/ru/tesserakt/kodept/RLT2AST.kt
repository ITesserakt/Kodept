package ru.tesserakt.kodept

import arrow.core.*
import org.jetbrains.annotations.TestOnly
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.core.Internal
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.lexer.ExpressionToken

private fun RLT.Assignment.expandCompound(left: AST.Lvalue, right: AST.Expression, token: String) = when (token) {
    ExpressionToken.PLUS_EQUALS.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Add).andThen { it.withRLT() }
    ExpressionToken.SUB_EQUALS.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Sub).andThen { it.withRLT() }
    ExpressionToken.TIMES_EQUALS.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Mul)
        .andThen { it.withRLT() }
    ExpressionToken.DIV_EQUALS.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Div).andThen { it.withRLT() }
    ExpressionToken.MOD_EQUALS.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Mod).andThen { it.withRLT() }
    ExpressionToken.POW_EQUALS.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Pow).andThen { it.withRLT() }
    ExpressionToken.OR_LOGIC_EQUALS.name -> AST::Logical.partially3(AST.Logical.Kind.Disjunction)
        .andThen { it.withRLT() }
    ExpressionToken.AND_LOGIC_EQUALS.name -> AST::Logical.partially3(AST.Logical.Kind.Conjunction)
        .andThen { it.withRLT() }
    ExpressionToken.OR_BIT_EQUALS.name -> AST::Binary.partially3(AST.Binary.Kind.Or).andThen { it.withRLT() }
    ExpressionToken.AND_BIT_EQUALS.name -> AST::Binary.partially3(AST.Binary.Kind.And).andThen { it.withRLT() }
    ExpressionToken.XOR_BIT_EQUALS.name -> AST::Binary.partially3(AST.Binary.Kind.Xor).andThen { it.withRLT() }
    ExpressionToken.EQUALS.name -> { _, r -> r }
    else -> throw IllegalStateException("Impossible")
}.let {
    fun RLT.Lvalue.copyLvalue(value: AST.Lvalue): AST.Expression = when (value) {
        is AST.Dereference -> value.copy().withRLT()
        is AST.FunctionCall -> value.copy().withRLT()
        is AST.Reference -> value.copy().withRLT()
        is AST.TypeReference -> value.copy().withRLT()
    }

    AST.Assignment(left, it(lvalue.copyLvalue(left), right)).withRLT()
}

context (RLT.BinaryOperation)
        private fun expandBinary(left: AST.Expression, right: AST.Expression, token: String) = when (token) {
    ExpressionToken.DOT.name -> AST::Dereference
    ExpressionToken.PLUS.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Add)
    ExpressionToken.SUB.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Sub)
    ExpressionToken.TIMES.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Mul)
    ExpressionToken.DIV.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Div)
    ExpressionToken.MOD.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Mod)
    ExpressionToken.POW.name -> AST::Mathematical.partially3(AST.Mathematical.Kind.Pow)
    ExpressionToken.OR_BIT.name -> AST::Binary.partially3(AST.Binary.Kind.Or)
    ExpressionToken.AND_BIT.name -> AST::Binary.partially3(AST.Binary.Kind.And)
    ExpressionToken.XOR_BIT.name -> AST::Binary.partially3(AST.Binary.Kind.Xor)
    ExpressionToken.OR_LOGIC.name -> AST::Logical.partially3(AST.Logical.Kind.Disjunction)
    ExpressionToken.AND_LOGIC.name -> AST::Logical.partially3(AST.Logical.Kind.Conjunction)
    ExpressionToken.LESS.name -> AST::Comparison.partially3(AST.Comparison.Kind.Less)
    ExpressionToken.LESS_EQUALS.name -> AST::Comparison.partially3(AST.Comparison.Kind.LessEqual)
    ExpressionToken.EQUIV.name -> AST::Comparison.partially3(AST.Comparison.Kind.Equal)
    ExpressionToken.NOT_EQUIV.name -> AST::Comparison.partially3(AST.Comparison.Kind.NonEqual)
    ExpressionToken.GREATER_EQUALS.name -> AST::Comparison.partially3(AST.Comparison.Kind.GreaterEqual)
    ExpressionToken.GREATER.name -> AST::Comparison.partially3(AST.Comparison.Kind.Greater)
    ExpressionToken.SPACESHIP.name -> AST::Comparison.partially3(AST.Comparison.Kind.Complex)
    ExpressionToken.ELVIS.name -> AST::Elvis
    else -> throw IllegalStateException("Impossible")
}(left, right).withRLT()

private fun expandUnary(expression: AST.Expression, token: String) = when (token) {
    ExpressionToken.PLUS.name -> AST.Absolution(expression)
    ExpressionToken.SUB.name -> AST.Negation(expression)
    ExpressionToken.NOT_BIT.name -> AST.BitInversion(expression)
    ExpressionToken.NOT_LOGIC.name -> AST.Inversion(expression)
    else -> throw IllegalStateException("Impossible")
}

private fun RLT.Context.convert(): AST.ResolutionContext = unfold().let { (fromRoot, chain) ->
    AST.ResolutionContext(fromRoot != null, chain.map { (it.convert() as AST.TypeReference).type })
}

private fun RLT.UserSymbol.Type.convert() = AST.Type(text.value()).withRLT()

private fun RLT.UserSymbol.Identifier.convert() = with(RLT.Reference(this)) {
    AST.Reference(text.value()).withRLT()
}

private fun RLT.Literal.convert(): AST.Literal = when (this) {
    is RLT.Literal.Floating -> when {
        '.' in text.value() || text.value().contains('e', true) -> AST.FloatingLiteral(
            text.value().toBigDecimal()
        )

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

private fun RLT.MaybeTypedParameter.convert() = AST.InferredParameter(id.text.value(), type?.convert()).withRLT()

private fun RLT.MaybeTypedParameterTuple.convert() = params.map { it.convert() }

private fun RLT.TypedParameter.convert() = AST.Parameter(id.text.value(), type.convert()).withRLT()

private fun RLT.TypedParameterTuple.convert() = params.map { it.convert() }

private fun RLT.If.Else.convert() = AST.IfExpr.ElseExpr(body.convert()).withRLT()

private fun RLT.If.Elif.convert() = AST.IfExpr.ElifExpr(condition.convert(), body.convert()).withRLT()

private fun RLT.Function.Bodied.convert() = AST.FunctionDecl(
    id.text.value(), params.flatMap { it.convert() }, returnType?.convert(), body.convert()
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

    is RLT.Struct -> AST.StructDecl(id.text.value(), varsToAlloc.map { it.convert() }, rest.map { it.convert() })
        .withRLT()

    is RLT.Trait -> AST.TraitDecl(id.text.value(), rest.map { it.convert() }).withRLT()

    is RLT.ForeignType -> AST.ForeignStructDecl(id.text.value(), type.text.value().removeSurrounding("\"")).withRLT()

    is RLT.Function.Foreign -> AST.ForeignFunctionDecl(
        id.text.value(),
        params.flatMap { it.convert() },
        returnType?.convert() as? AST.TypeReference,
        descriptor.text.value().removeSurrounding("\"")
    ).withRLT()
}

private fun RLT.StructLevelNode.convert() = when (this) {
    is RLT.Function.Bodied -> convert()
}

private fun RLT.ObjectLevelNode.convert(): AST.TraitLevel = when (this) {
    is RLT.Function.Bodied -> convert()

    is RLT.Function.Abstract -> AST.AbstractFunctionDecl(
        id.text.value(), params.flatMap { it.convert() }, returnType?.convert()
    ).withRLT()
}

private fun RLT.BlockLevelNode.convert(): AST.BlockLevel = when (this) {
    is RLT.StatementNode -> convert()
    is RLT.ExpressionNode -> convert()
}

private fun RLT.StatementNode.convert(): AST.BlockLevel = when (this) {
    is RLT.Assignment -> when (this) {
        is RLT.CompoundAssignment -> expandCompound(
            lvalue.convert(), expression.convert(), compoundOperator.type
        )

        is RLT.InitializedAssignment -> AST.InitializedVar(
            lvalue.id.convert(), lvalue is RLT.Variable.Mutable, lvalue.type?.convert(), expression.convert()
        ).withRLT()

        else -> expandCompound(
            lvalue.convert(), expression.convert(), equals.type
        )
    }

    is RLT.Function.Bodied -> convert()

    is RLT.Variable -> throw IllegalStateException("Thrown out")
}

private fun RLT.Body.Block.convert(): AST.Expression = when (block.size) {
    0 -> AST.ExpressionList(nonEmptyListOf(AST.TupleLiteral.unit.withRLT())).withRLT()
    1 -> {
        val head = block.first()
        if (head is RLT.ExpressionNode) head.convert()
        else AST.ExpressionList(nonEmptyListOf(head.convert(), AST.TupleLiteral.unit.withRLT())).withRLT()
    }

    else -> AST.ExpressionList(NonEmptyList.fromListUnsafe(block.map { it.convert() })).withRLT()
}

private fun RLT.ExpressionNode.convert(): AST.Expression = when (this) {
    is RLT.BinaryOperation -> expandBinary(left.convert(), right.convert(), op.type)

    is RLT.Body.Block -> convert()

    is RLT.Body.Expression -> AST.ExpressionList(nonEmptyListOf(expression.convert())).withRLT()

    is RLT.If -> AST.IfExpr(
        condition.convert(), body.convert(), elifs.map { it.convert() }, el?.convert()
    ).withRLT()

    is RLT.UnaryOperation -> expandUnary(expression.convert(), op.type).withRLT()

    is RLT.While -> AST.WhileExpr(condition.convert(), body.convert()).withRLT()

    is RLT.Parameter -> id.convert()

    is RLT.TermNode -> convert()

    is RLT.Literal -> convert()
    is RLT.ParameterTuple -> convert()
}

private fun RLT.Lvalue.convert(): AST.Lvalue = when (this) {
    is RLT.Variable -> id.convert()
    is RLT.TermNode -> convert()
}

private fun RLT.Reference.convert(): AST.Lvalue = when (this) {
    is RLT.ContextualReference -> when (val r = ref) {
        is RLT.UserSymbol.Identifier -> AST.Reference(ref.text.value(), context.convert()).withRLT()
        is RLT.UserSymbol.Type -> AST.TypeReference(r.convert(), context.convert()).withRLT()
    }

    else -> when (val r = ref) {
        is RLT.UserSymbol.Identifier -> AST.Reference(ref.text.value()).withRLT()
        is RLT.UserSymbol.Type -> AST.TypeReference(r.convert()).withRLT()
    }
}

private fun RLT.TermNode.convert(): AST.Lvalue = when (this) {
    is RLT.Application -> AST.FunctionCall(
        expr.convert() as AST.Reference,
        params.map(RLT.ParameterTuple::convert).filterNot { it.items.isEmpty() }).withRLT()

    is RLT.ContextualReference -> when (val r = ref) {
        is RLT.UserSymbol.Identifier -> AST.Reference(ref.text.value(), context.convert()).withRLT()
        is RLT.UserSymbol.Type -> AST.TypeReference(r.convert(), context.convert()).withRLT()
    }

    is RLT.Reference -> convert()
}

fun RLT.TypeNode.convert(): AST.TypeLike = when (this) {
    is RLT.TupleType -> AST.TupleType(types.map { it.convert() }).withRLT()
    is RLT.UnionType -> AST.UnionType(types.map { it.convert() }).withRLT()
    is RLT.ContextualReference -> AST.TypeReference(
        with(ref) { AST.Type(ref.text.value()).withRLT() }, context.convert()
    ).withRLT()

    is RLT.Reference -> AST.TypeReference(with(ref) { AST.Type(ref.text.value()).withRLT() }, null).withRLT()
}

@OptIn(Internal::class)
fun RLT.File.convert(): AST.FileDecl = AST.FileDecl(moduleList.map(RLT.Module::convert)).withRLT()

@Suppress("KotlinConstantConditions")
@Internal
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
    is RLT.UserSymbol.Type, is RLT.MaybeTypedParameterTuple, is RLT.Symbol, is RLT.Keyword, is RLT.UserSymbol.Identifier -> throw IllegalStateException(
        "Thrown out"
    )

    is RLT.InitializedAssignment -> convert()
}