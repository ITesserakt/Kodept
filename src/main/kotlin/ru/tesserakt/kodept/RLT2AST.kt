package ru.tesserakt.kodept

import arrow.core.NonEmptyList
import arrow.core.nonEmptyListOf
import arrow.core.partially2
import org.jetbrains.annotations.TestOnly
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.lexer.ExpressionToken

private fun RLT.Assignment.expandCompound(left: AST.Lvalue, right: AST.Expression, token: String) = when (token) {
    ExpressionToken.PLUS_EQUALS.name -> { l: AST.Cell<AST.Expression>, r: AST.Cell<AST.Expression> ->
        AST.Mathematical(
            l, r, AST.Mathematical.Kind.Add
        ).withRLT()
    }
    ExpressionToken.SUB_EQUALS.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Sub).withRLT() }
    ExpressionToken.TIMES_EQUALS.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Mul).withRLT() }
    ExpressionToken.DIV_EQUALS.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Div).withRLT() }
    ExpressionToken.MOD_EQUALS.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Mod).withRLT() }
    ExpressionToken.POW_EQUALS.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Pow).withRLT() }
    ExpressionToken.OR_LOGIC_EQUALS.name -> { l, r -> AST.Logical(l, r, AST.Logical.Kind.Disjunction).withRLT() }
    ExpressionToken.AND_LOGIC_EQUALS.name -> { l, r -> AST.Logical(l, r, AST.Logical.Kind.Conjunction).withRLT() }
    ExpressionToken.OR_BIT_EQUALS.name -> { l, r -> AST.Binary(l, r, AST.Binary.Kind.Or).withRLT() }
    ExpressionToken.AND_BIT_EQUALS.name -> { l, r -> AST.Binary(l, r, AST.Binary.Kind.And).withRLT() }
    ExpressionToken.XOR_BIT_EQUALS.name -> { l, r -> AST.Binary(l, r, AST.Binary.Kind.Xor).withRLT() }
    ExpressionToken.EQUALS.name -> { _, r -> r.value }
    else -> throw IllegalStateException("Impossible")
}.let {
    val expanded = it(left.new(), right.move())
    AST.Assignment(left, expanded).withRLT()
}

context (RLT.BinaryOperation)
        private fun expandBinary(left: AST.Expression, right: AST.Expression, token: String) = when (token) {
    ExpressionToken.PLUS.name -> { l: AST.Cell<AST.Expression>, r: AST.Cell<AST.Expression> ->
        AST.Mathematical(
            l, r, AST.Mathematical.Kind.Add
        ).withRLT()
    }
    ExpressionToken.SUB.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Sub) }
    ExpressionToken.TIMES.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Mul) }
    ExpressionToken.DIV.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Div) }
    ExpressionToken.MOD.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Mod) }
    ExpressionToken.POW.name -> { l, r -> AST.Mathematical(l, r, AST.Mathematical.Kind.Pow) }
    ExpressionToken.OR_BIT.name -> { l, r -> AST.Binary(l, r, AST.Binary.Kind.Or) }
    ExpressionToken.AND_BIT.name -> { l, r -> AST.Binary(l, r, AST.Binary.Kind.And) }
    ExpressionToken.XOR_BIT.name -> { l, r -> AST.Binary(l, r, AST.Binary.Kind.Xor) }
    ExpressionToken.OR_LOGIC.name -> { l, r -> AST.Logical(l, r, AST.Logical.Kind.Disjunction) }
    ExpressionToken.AND_LOGIC.name -> { l, r -> AST.Logical(l, r, AST.Logical.Kind.Conjunction) }
    ExpressionToken.LESS.name -> { l, r -> AST.Comparison(l, r, AST.Comparison.Kind.Less) }
    ExpressionToken.LESS_EQUALS.name -> { l, r -> AST.Comparison(l, r, AST.Comparison.Kind.LessEqual) }
    ExpressionToken.EQUIV.name -> { l, r -> AST.Comparison(l, r, AST.Comparison.Kind.Equal) }
    ExpressionToken.NOT_EQUIV.name -> { l, r -> AST.Comparison(l, r, AST.Comparison.Kind.NonEqual) }
    ExpressionToken.GREATER_EQUALS.name -> { l, r -> AST.Comparison(l, r, AST.Comparison.Kind.GreaterEqual) }
    ExpressionToken.GREATER.name -> { l, r -> AST.Comparison(l, r, AST.Comparison.Kind.Greater) }
    ExpressionToken.SPACESHIP.name -> { l, r -> AST.Comparison(l, r, AST.Comparison.Kind.Complex) }
    else -> throw IllegalStateException("Impossible")
}(left.move(), right.move()).withRLT()

private fun expandUnary(expression: AST.Expression, token: String) = when (token) {
    ExpressionToken.PLUS.name -> AST.Absolution(expression.move())
    ExpressionToken.SUB.name -> AST.Negation(expression.move())
    ExpressionToken.NOT_BIT.name -> AST.BitInversion(expression.move())
    ExpressionToken.NOT_LOGIC.name -> AST.Inversion(expression.move())
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

    is RLT.Literal.Tuple -> AST.TupleLiteral(expressions.map(RLT.ExpressionNode::convert).move()).withRLT()
}

private fun RLT.Module.convert(): AST.ModuleDecl = when (this) {
    is RLT.Module.Global -> AST::ModuleDecl.partially2(true)
    is RLT.Module.Ordinary -> AST::ModuleDecl.partially2(false)
}.invoke(id.text.value(), rest.map(RLT.TopLevelNode::convert).move()).withRLT()

private fun RLT.ParameterTuple.convert(): AST.TupleLiteral =
    AST.TupleLiteral(params.map(RLT.Parameter::convert).move()).withRLT()

private fun RLT.MaybeTypedParameter.convert() =
    AST.InferredParameter(id.text.value(), type?.convert()?.move()).withRLT()

private fun RLT.MaybeTypedParameterTuple.convert() = params.map { it.convert() }

private fun RLT.TypedParameter.convert() = AST.Parameter(id.text.value(), type.convert().move()).withRLT()

private fun RLT.TypedParameterTuple.convert() = params.map { it.convert() }

private fun RLT.If.Else.convert() = AST.IfExpr.ElseExpr(body.convert().move()).withRLT()

private fun RLT.If.Elif.convert() = AST.IfExpr.ElifExpr(condition.convert().move(), body.convert().move()).withRLT()

private fun RLT.Match.Branch.convert() = AST.IfExpr.ElifExpr(condition.convert(), body.convert()).withRLT()

private fun RLT.Match.convert(): AST.IfExpr {
    fun transformReceiver(receiver: AST.Expression, pattern: AST.Expression) = with(pattern.rlt) {
        AST.Comparison(receiver, pattern, AST.Comparison.Kind.Equal).withRLT()
    }

    return if (receiver == null) AST.IfExpr(
        branches.head.condition.convert(), branches.head.body.convert(), branches.tail.map { it.convert() }, null
    ).withRLT()
    else AST.IfExpr(
        transformReceiver(receiver!!.convert(), branches.head.condition.convert()),
        branches.head.body.convert(),
        branches.tail.map {
            it.convert().run { copy(conditionCell = transformReceiver(receiver!!.convert(), condition).move()) }
        },
        null
    ).withRLT()
}

private fun RLT.Function.Bodied.convert() = AST.FunctionDecl(
    id.text.value(), params.flatMap { it.convert() }.move(), returnType?.convert()?.move(), body.convert().move()
).withRLT()

private fun RLT.Enum.convert() = when (this) {
    is RLT.Enum.Heap -> AST::EnumDecl.partially2(false)
    is RLT.Enum.Stack -> AST::EnumDecl.partially2(true)
}.invoke(id.text.value(), rest.map {
    with(it) { AST.EnumDecl.Entry(it.text.value()).withRLT().move() }
}).withRLT()

private fun RLT.TopLevelNode.convert(): AST.TopLevel = when (this) {
    is RLT.Function.Bodied -> convert()

    is RLT.Enum -> convert()

    is RLT.Struct -> AST.StructDecl(
        id.text.value(),
        varsToAlloc.map { it.convert() }.move(),
        rest.map { it.convert() }.move()
    ).withRLT()

    is RLT.Trait -> AST.TraitDecl(id.text.value(), rest.map { it.convert() }.move()).withRLT()

    is RLT.ForeignType -> AST.ForeignStructDecl(id.text.value(), type.text.value().removeSurrounding("\"")).withRLT()

    is RLT.Function.Foreign -> AST.ForeignFunctionDecl(
        id.text.value(),
        params.flatMap { it.convert() }.move(),
        (returnType?.convert() as? AST.TypeReference)?.move(),
        descriptor.text.value().removeSurrounding("\"")
    ).withRLT()

    is RLT.Extension -> AST.ExtensionDecl(
        onType.convert() as AST.TypeReference,
        forTrait.convert() as AST.TypeReference,
        body.map { it.convert() })
}

private fun RLT.StructLevelNode.convert() = when (this) {
    is RLT.Function.Bodied -> convert()
}

private fun RLT.ObjectLevelNode.convert(): AST.TraitLevel = when (this) {
    is RLT.Function.Bodied -> convert()

    is RLT.Function.Abstract -> AST.AbstractFunctionDecl(
        id.text.value(), params.flatMap { it.convert() }.move(), returnType?.convert()?.move()
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
            lvalue.id.convert().move(),
            lvalue is RLT.Variable.Mutable,
            lvalue.type?.convert()?.move(),
            expression.convert().move()
        ).withRLT()

        else -> expandCompound(
            lvalue.convert(), expression.convert(), equals.type
        )
    }

    is RLT.Function.Bodied -> convert()

    is RLT.Variable -> throw IllegalStateException("Thrown out")

    is RLT.While -> AST.WhileExpr(condition.convert().move(), body.convert().move()).withRLT()
}

private fun RLT.Body.Block.convert(): AST.Expression = when (block.size) {
    0 -> AST.ExpressionList(nonEmptyListOf(AST.TupleLiteral.unit.withRLT().move())).withRLT()
    1 -> {
        val head = block.first()
        if (head is RLT.ExpressionNode) head.convert()
        else AST.ExpressionList(nonEmptyListOf(head.convert().move(), AST.TupleLiteral.unit.withRLT().move())).withRLT()
    }

    else -> AST.ExpressionList(NonEmptyList.fromListUnsafe(block.map { it.convert().move() })).withRLT()
}

private fun RLT.ExpressionNode.convert(): AST.Expression = when (this) {
    is RLT.Access -> AST.Dereference(left.convert().move(), right.convert().move()).withRLT()

    is RLT.BinaryOperation -> expandBinary(left.convert(), right.convert(), op.type)

    is RLT.Body.Block -> convert()

    is RLT.Body.Expression -> AST.ExpressionList(nonEmptyListOf(expression.convert().move())).withRLT()

    is RLT.If -> AST.IfExpr(
        condition.convert().move(), body.convert().move(), elifs.map { it.convert().move() }, el?.convert()?.move()
    ).withRLT()

    is RLT.UnaryOperation -> expandUnary(expression.convert(), op.type).withRLT()

    is RLT.Parameter -> id.convert()

    is RLT.TermNode -> convert()

    is RLT.Literal -> convert()
    is RLT.ParameterTuple -> convert()
    is RLT.Lambda -> AST.LambdaExpr(params.map {
        with(it) { AST.InferredParameter(it.text.value()).withRLT().move() }
    }, body.convert().move(), null).withRLT()

    is RLT.Match -> convert()

    is RLT.Application -> AST.FunctionCall(
        expr.convert().move(),
        params.map { it.convert() }.move()
    ).withRLT()
}

private fun RLT.Lvalue.convert(): AST.Lvalue = when (this) {
    is RLT.Variable -> id.convert()
    is RLT.TermNode -> convert()
}

private fun RLT.Reference.convert(): AST.Lvalue = when (this) {
    is RLT.ContextualReference -> when (val r = ref) {
        is RLT.UserSymbol.Identifier -> AST.Reference(ref.text.value(), context.convert()).withRLT()
        is RLT.UserSymbol.Type -> AST.TypeReference(r.convert().move(), context.convert()).withRLT()
    }

    else -> when (val r = ref) {
        is RLT.UserSymbol.Identifier -> AST.Reference(ref.text.value()).withRLT()
        is RLT.UserSymbol.Type -> AST.TypeReference(r.convert().move()).withRLT()
    }
}

private fun RLT.TermNode.convert(): AST.Lvalue = when (this) {
    is RLT.ContextualReference -> when (val r = ref) {
        is RLT.UserSymbol.Identifier -> AST.Reference(ref.text.value(), context.convert()).withRLT()
        is RLT.UserSymbol.Type -> AST.TypeReference(r.convert().move(), context.convert()).withRLT()
    }

    is RLT.Reference -> convert()
}

fun RLT.TypeNode.convert(): AST.TypeLike = when (this) {
    is RLT.TupleType -> AST.TupleType(types.map { it.convert() }.move()).withRLT()
    is RLT.UnionType -> AST.UnionType(types.map { it.convert().move() }).withRLT()
    is RLT.ContextualReference -> AST.TypeReference(
        with(ref) { AST.Type(ref.text.value()).withRLT().move() }, context.convert()
    ).withRLT()

    is RLT.Reference -> AST.TypeReference(with(ref) { AST.Type(ref.text.value()).withRLT().move() }, null).withRLT()
}

@OptIn(Internal::class)
fun RLT.File.convert(): AST.FileDecl = AST.FileDecl(moduleList.map { it.convert().move() }).withRLT()

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
    is RLT.Match.Branch -> convert()
    is RLT.File -> convert()
    is RLT.Module -> convert()
    is RLT.MaybeTypedParameter -> convert()
    is RLT.UserSymbol.Type, is RLT.MaybeTypedParameterTuple, is RLT.Symbol, is RLT.Keyword, is RLT.UserSymbol.Identifier -> throw IllegalStateException(
        "Thrown out"
    )

    is RLT.InitializedAssignment -> convert()
}