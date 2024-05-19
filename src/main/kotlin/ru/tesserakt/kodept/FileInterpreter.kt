package ru.tesserakt.kodept

import arrow.core.Either
import arrow.core.left
import arrow.core.right
import ru.tesserakt.kodept.ExpressionResult.*
import ru.tesserakt.kodept.ExpressionResult.Companion.tryParse
import ru.tesserakt.kodept.ExpressionResult.Number
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.traversal.BinaryOperatorDesugaring
import ru.tesserakt.kodept.traversal.ReferenceResolver
import ru.tesserakt.kodept.traversal.TypeReferenceResolver
import ru.tesserakt.kodept.traversal.UnaryOperatorDesugaring
import ru.tesserakt.kodept.traversal.inference.mangle
import java.math.BigDecimal
import java.math.BigInteger

sealed interface ExpressionResult {
    data class Number(val value: BigInteger) : ExpressionResult
    data class Floating(val value: BigDecimal) : ExpressionResult
    data class Char(val value: kotlin.Char) : ExpressionResult
    data class String(val value: kotlin.String) : ExpressionResult
    data class Bool(val value: Boolean) : ExpressionResult
    data class Struct(val items: Map<kotlin.String, ExpressionResult>) : ExpressionResult
    data class Function(val fn: Either<AST.FunctionDecl, AST.LambdaExpr>) : ExpressionResult
    object Empty : ExpressionResult

    companion object {
        val unit = Struct(emptyMap())

        internal fun Any?.tryParse() = when (this) {
            is BigInteger -> Number(this)
            is BigDecimal -> Floating(this)
            is kotlin.Char -> Char(this)
            is kotlin.String -> String(this)
            is Unit -> unit
            is ExpressionResult -> this
            is Boolean -> Bool(this)
            else -> null
        }
    }
}

data class ProgramState(
    val input: List<String>,
    val output: Int,
    val variables: Map<String, ExpressionResult>,
    val result: ExpressionResult,
    val mainFound: Boolean,
)

class FileInterpreter : Interpreter<ProgramState, AST.Node, List<String>> {
    private fun Either<AST.FunctionDecl, AST.LambdaExpr>.join(
        state: ProgramState,
        call: AST.FunctionCall,
    ): ProgramState {
        val newState = call.params.runningFold(state) { acc, next -> next.eval(acc) }
        val map = fold({ it.params }, { it.params }).zip(newState.drop(1)).associate {
            it.first.mangle() to it.second.result
        }
        val last = newState.last()
        return fold({ it.rest }, { it.body }).eval(last.copy(variables = last.variables + map))
            .copy(variables = last.variables)
    }

    private fun AST.Expression.eval(state: ProgramState): ProgramState = when (this) {
        is AST.BinaryOperator -> BinaryOperatorDesugaring.contract()
        is AST.UnaryOperator -> UnaryOperatorDesugaring.contract()
        is AST.ExpressionList -> expressions.fold(state) { acc, next -> join(acc, next) }
        is AST.IfExpr -> {
            val condition = condition.eval(state)
            if (condition.result is Bool && condition.result.value) body.eval(condition)
            else {
                val passing = elifs.map { it to it.condition.eval(condition) }
                    .firstNotNullOfOrNull { (node, s) -> node.takeIf { (s.result as? Bool)?.value == true } }
                passing?.body?.eval(state) ?: el?.body?.eval(state) ?: state.copy(result = Empty)
            }
        }

        is AST.BinaryLiteral -> state.copy(result = Number(value))
        is AST.CharLiteral -> state.copy(result = Char(value))
        is AST.DecimalLiteral -> state.copy(result = Number(value))
        is AST.FloatingLiteral -> state.copy(result = Floating(value))
        is AST.HexLiteral -> state.copy(result = Number(value))
        is AST.OctalLiteral -> state.copy(result = Number(value))
        is AST.StringLiteral -> state.copy(result = String(value))
        is AST.TupleLiteral -> if (this == AST.TupleLiteral.unit) state.copy(result = ExpressionResult.unit) else state.copy(
            result = Struct(this.items.asSequence().runningFold(state) { acc, expr -> expr.eval(acc) }.map { it.result }
                .drop(1).withIndex().associate { it.index.toString() to it.value })
        )

        is AST.FunctionCall -> {
            when (val r = reference) {
                is AST.ResolvedReference -> when (val ref = r.referral) {
                    is AST.AbstractFunctionDecl -> TODO()
                    is AST.ForeignFunctionDecl -> {
                        fun ExpressionResult.tryConvert(): Any = when (this) {
                            is Bool -> value
                            is ExpressionResult.Char -> value
                            Empty -> error("Wrong result: $this")
                            is ExpressionResult.Function -> fn.join(state, this@eval).result.tryConvert()
                            is Floating -> value
                            is Number -> value
                            is ExpressionResult.String -> value
                            is Struct -> items.mapValues { it.value.tryConvert() }
                        }

                        val newState = params.runningFold(state) { acc, next -> next.eval(acc) }
                        require(ref.hasAction) { "Declaration `${ref.name}` has no action" }
                        val value =
                            ref.action.action(newState.filter { it.result !is Empty }.map { it.result.tryConvert() })
                        newState.last()
                            .copy(result = value.tryParse() ?: throw IllegalStateException("Unknown result: $value"))
                    }

                    is AST.FunctionDecl -> ref.left().join(state, this)

                    is AST.InferredParameter -> {
                        val fd = state.variables.getValue(r.mangle())
                        fd as ExpressionResult.Function
                        fd.fn.join(state, this)
                    }
                    is AST.InitializedVar -> TODO()
                }
                is AST.FunctionCall -> reference.eval(state)
                else -> TODO()
            }
        }

        is AST.ResolvedReference -> when (val r = referral) {
            is AST.AbstractFunctionDecl -> TODO()
            is AST.ForeignFunctionDecl -> TODO()
            is AST.FunctionDecl -> state.copy(result = Function(r.left()))
            is AST.InitializedVar -> state.copy(result = state.variables.getValue(mangle()))
            is AST.InferredParameter -> state.copy(result = state.variables.getValue(r.mangle()))
        }

        is AST.Reference -> ReferenceResolver.contract()
        is AST.TypeReference -> TypeReferenceResolver.contract()
        is AST.LambdaExpr -> state.copy(result = Function(this.right()))
        is AST.Intrinsics.AccessVariable -> {
            val self = state.variables.entries.first { it.key.startsWith("self") }.value
            state.copy(
                result = when (self) {
                    is Struct -> self.items.getValue(variable.mangle())
                    else -> throw IllegalStateException("Wrong result: $self")
                }
            )
        }
        is AST.Intrinsics.Construct -> {
            val computedParams = params.runningFold(state) { acc, next -> next.eval(acc) }.zip(params)
                .associate { it.second.mangle() to it.first.result }
            state.copy(result = Struct(computedParams))
        }
    }

    override fun initialState(input: List<String>) = ProgramState(input, 0, emptyMap(), Empty, false)

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
            expr.copy(variables = expr.variables + ((program.reference as AST.ResolvedReference).mangle() to expr.result))
        }

        is AST.EnumDecl.Entry -> state
        is AST.ForeignStructDecl -> state
        is AST.ModuleDecl -> program.rest.fold(state, ::join)
        is AST.EnumDecl -> state
        is AST.StructDecl -> program.rest.fold(state, ::join)
        is AST.TraitDecl -> program.rest.fold(state, ::join)
        is AST.Assignment -> when (val left = program.left) {
            is AST.FunctionCall -> TODO()
            is AST.ResolvedReference -> {
                val expr = program.right.eval(state)
                expr.copy(variables = expr.variables + (left.mangle() to expr.result))
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
        is AST.WhileExpr -> {
            var s = state
            fun AST.Expression.condIsTrue() = eval(s).let { it.result is Bool && it.result.value }

            while (program.condition.condIsTrue()) {
                s = program.body.eval(s)
            }
            s
        }
        is AST.Cell<*> -> join(state, program.value)
        is AST.ExtensionDecl -> program.rest.fold(state, ::join)
    }
}