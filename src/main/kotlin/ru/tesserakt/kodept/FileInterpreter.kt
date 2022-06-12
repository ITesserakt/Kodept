package ru.tesserakt.kodept

import arrow.core.andThen
import ru.tesserakt.kodept.core.AST
import java.math.BigInteger

data class ProgramState(
    val input: List<String>,
    val output: Int,
    val state: Map<AST.ResolvedReference, Any>,
    val result: Any?,
    val functionParameters: Map<AST.ParameterLike, Any?>,
    val mainFound: Boolean,
)

class FileInterpreter : Interpreter<ProgramState, AST.Node, List<String>> {
    private val comparisonOps = mapOf(
        AST.Comparison.Kind.Complex to BigInteger::compareTo,
        AST.Comparison.Kind.Equal to Any::equals,
        AST.Comparison.Kind.Greater to { l, r -> l > r },
        AST.Comparison.Kind.Less to { l, r -> l < r },
        AST.Comparison.Kind.GreaterEqual to { l, r -> l >= r },
        AST.Comparison.Kind.LessEqual to { l, r -> l <= r },
        AST.Comparison.Kind.NonEqual to (Any::equals andThen Boolean::not),
    )

    private val binaryOps = mapOf(
        AST.Binary.Kind.And to BigInteger::and,
        AST.Binary.Kind.Or to BigInteger::or,
        AST.Binary.Kind.Xor to { l, r -> l.xor(r) },
    )

    private val logicalOps = mapOf(
        AST.Logical.Kind.Conjunction to Boolean::and,
        AST.Logical.Kind.Disjunction to Boolean::or
    )

    private val mathOps = mapOf(
        AST.Mathematical.Kind.Add to BigInteger::add,
        AST.Mathematical.Kind.Div to BigInteger::div,
        AST.Mathematical.Kind.Mod to BigInteger::mod,
        AST.Mathematical.Kind.Mul to BigInteger::multiply,
        AST.Mathematical.Kind.Pow to { l, r -> l.pow(r.toInt()) },
        AST.Mathematical.Kind.Sub to BigInteger::subtract
    )

    private inline fun <reified T, reified K : AST.BinaryOperator.OperatorKind> AST.BinaryOperator.evalBinaryOperatorUsing(
        ops: Map<K, (T, T) -> Any?>,
        state: ProgramState,
    ): ProgramState {
        require(kind is K)

        val left = left.eval(state)
        val right = right.eval(left)
        val res = ops[kind]!!.invoke(left.result as T, right.result as T)
        return right.copy(result = res)
    }

    private fun AST.Expression.eval(state: ProgramState): ProgramState = when (this) {
        is AST.Binary -> evalBinaryOperatorUsing(binaryOps, state)
        is AST.Comparison -> evalBinaryOperatorUsing(comparisonOps, state)
        is AST.Dereference -> {
            val evalLeft = left.eval(state)
            val newState = evalLeft.copy(
                functionParameters = evalLeft.functionParameters + (AST.InferredParameter(
                    "self",
                    null
                ) to evalLeft.result)
            )
            right.eval(newState).copy(functionParameters = evalLeft.functionParameters)
        }

        is AST.Elvis -> TODO()
        is AST.Logical -> evalBinaryOperatorUsing(logicalOps, state)
        is AST.Mathematical -> evalBinaryOperatorUsing(mathOps, state)
        is AST.ExpressionList -> expressions.fold(state) { acc, next -> join(acc, next) }
        is AST.IfExpr -> {
            val condition = condition.eval(state)
            if (condition.result as Boolean)
                body.eval(condition)
            else {
                val passing = elifs.map { it to it.condition.eval(condition) }
                    .firstNotNullOfOrNull { (node, s) -> node.takeIf { s.result as? Boolean == true } }
                passing?.body?.eval(state) ?: el?.body?.eval(state) ?: state.copy(result = null)
            }
        }

        is AST.BinaryLiteral -> state.copy(result = value)
        is AST.CharLiteral -> state.copy(result = value)
        is AST.DecimalLiteral -> state.copy(result = value)
        is AST.FloatingLiteral -> state.copy(result = value)
        is AST.HexLiteral -> state.copy(result = value)
        is AST.OctalLiteral -> state.copy(result = value)
        is AST.StringLiteral -> state.copy(result = value)
        is AST.Stub -> state
        is AST.TupleLiteral -> if (this == AST.TupleLiteral.unit) state.copy(result = Unit) else state.copy(
            result = this.items.runningFold(
                state
            ) { acc, expr -> expr.eval(acc) }.map { it.result }.drop(1)
        )

        is AST.FunctionCall -> {
            val r = reference as AST.ResolvedReference
            when (val ref = r.referral) {
                is AST.AbstractFunctionDecl -> TODO()
                is AST.ForeignFunctionDecl -> {
                    val newState = params.runningFold(state) { acc, next -> next.eval(acc) }
                    require(ref.hasAction) { "Declaration `${ref.name}` has no action" }
                    val value =
                        ref.action.action(newState.flatMap { it.result as? List<*> ?: listOfNotNull(it.result) })
                    newState.last().copy(result = value)
                }

                is AST.FunctionDecl -> {
                    val newState = params.runningFold(state) { acc, next -> next.eval(acc) }
                    val stateWithParams = state
                        .copy(functionParameters = newState.last().functionParameters + ref.params.zip(newState.flatMap {
                            it.result as? List<*> ?: emptyList()
                        }).toMap())
                    ref.rest.eval(stateWithParams).copy(functionParameters = newState.last().functionParameters)
                }

                is AST.InferredParameter -> TODO()
                is AST.InitializedVar -> TODO()
                is AST.Parameter -> TODO()
            }
        }

        is AST.ResolvedReference -> when (val r = referral) {
            is AST.AbstractFunctionDecl -> TODO()
            is AST.ForeignFunctionDecl -> TODO()
            is AST.FunctionDecl -> TODO()
            is AST.InitializedVar -> state.copy(result = state.state.getValue(this))
            is AST.ParameterLike -> state.copy(result = state.functionParameters.getValue(r))
        }

        is AST.Reference -> throw IllegalStateException(name)
        is AST.TypeReference -> TODO()
        is AST.Absolution -> state.copy(result = expr.eval(state).result)
        is AST.BitInversion -> state.copy(result = (expr.eval(state).result as BigInteger).inv())
        is AST.Inversion -> state.copy(result = !(expr.eval(state).result as Boolean))
        is AST.Negation -> state.copy(result = (expr.eval(state).result as BigInteger).negate())
        is AST.WhileExpr -> {
            var s = state
            val list = buildList {
                while (condition.eval(s).result as Boolean) {
                    s = body.eval(s)
                    add(s.result)
                }
            }
            s.copy(result = list)
        }
    }

    override fun initialState(input: List<String>) = ProgramState(input, 0, emptyMap(), null, emptyMap(), false)

    override fun join(state: ProgramState, program: AST.Node): ProgramState = when (program) {
        is AST.Expression -> program.eval(state)
        is AST.AbstractFunctionDecl -> state
        is AST.ForeignFunctionDecl -> state
        is AST.FunctionDecl -> {
            if (program.name == "main") {
                program.rest.eval(state).let {
                    it.copy(output = (it.result as? BigInteger)?.toInt() ?: 0, mainFound = true)
                }
            } else state
        }

        is AST.InferredParameter -> state
        is AST.InitializedVar -> {
            val expr = program.expr.eval(state)
            expr.copy(state = expr.state + (program.reference as AST.ResolvedReference to expr.result!!))
        }

        is AST.Parameter -> state
        is AST.EnumDecl.Entry -> state
        is AST.ForeignStructDecl -> state
        is AST.ModuleDecl -> program.rest.fold(state, ::join)
        is AST.EnumDecl -> state
        is AST.StructDecl -> program.rest.fold(state, ::join)
        is AST.TraitDecl -> program.rest.fold(state, ::join)
        is AST.Assignment -> when (val left = program.left) {
            is AST.Dereference -> TODO()
            is AST.FunctionCall -> TODO()
            is AST.ResolvedReference -> {
                val expr = program.right.eval(state)
                expr.copy(state = expr.state + (left to expr.result!!))
            }

            is AST.Reference -> throw IllegalStateException()
            is AST.TypeReference -> state
        }

        is AST.IfExpr.ElifExpr -> state
        is AST.IfExpr.ElseExpr -> state
        is AST.FileDecl -> program.modules.fold(state, ::join)
        is AST.TupleType -> state
        is AST.Type -> state
        is AST.UnionType -> state
    }
}