package ru.tesserakt.kodept

import ru.tesserakt.kodept.core.AST
import java.math.BigDecimal
import java.math.BigInteger

sealed interface ExpressionResult {
    data class Number(val value: BigInteger) : ExpressionResult
    data class Floating(val value: BigDecimal) : ExpressionResult
    data class Function(val params: List<AST.InferredParameter>, val body: ProgramState) : ExpressionResult
    data class Enum(val branches: List<Struct>) : ExpressionResult
    data class Struct(val items: List<ExpressionResult>) : ExpressionResult
}

data class ProgramState(
    val input: List<String>,
    val output: Int,
    val variables: Map<AST.ResolvedReference, Any>,
    val result: ExpressionResult?,
    val functionParameters: Map<AST.InferredParameter, Any?>,
    val mainFound: Boolean,
)

class FileInterpreter : Interpreter<ProgramState, AST.Node, List<String>> {
//    private fun AST.Expression.eval(state: ProgramState): ProgramState = when (this) {
//        is AST.BinaryOperator -> BinaryOperatorDesugaring.contract()
//        is AST.UnaryOperator -> UnaryOperatorDesugaring.contract()
//        is AST.Dereference -> DereferenceEliminator.contract()
//        is AST.ExpressionList -> expressions.fold(state) { acc, next -> join(acc, next) }
//        is AST.IfExpr -> {
//            val condition = condition.eval(state)
//            if (condition.result is ExpressionResult.Enum)
//                body.eval(condition)
//            else {
//                val passing = elifs.map { it to it.condition.eval(condition) }
//                    .firstNotNullOfOrNull { (node, s) -> node.takeIf { s.result as? Boolean == true } }
//                passing?.body?.eval(state) ?: el?.body?.eval(state) ?: state.copy(result = null)
//            }
//        }
//
//        is AST.BinaryLiteral -> state.copy(result = ExpressionResult.Number(value))
//        is AST.CharLiteral -> state.copy(result = )
//        is AST.DecimalLiteral -> state.copy(result = value)
//        is AST.FloatingLiteral -> state.copy(result = value)
//        is AST.HexLiteral -> state.copy(result = value)
//        is AST.OctalLiteral -> state.copy(result = value)
//        is AST.StringLiteral -> state.copy(result = value)
//        is AST.TupleLiteral -> if (this == AST.TupleLiteral.unit) state.copy(result = Unit) else state.copy(
//            result = this.items.runningFold(
//                state
//            ) { acc, expr -> expr.eval(acc) }.map { it.result }.drop(1)
//        )
//
//        is AST.FunctionCall -> {
//            val r = reference as AST.ResolvedReference
//            when (val ref = r.referral) {
//                is AST.AbstractFunctionDecl -> TODO()
//                is AST.ForeignFunctionDecl -> {
//                    val newState = params.runningFold(state) { acc, next -> next.eval(acc) }
//                    require(ref.hasAction) { "Declaration `${ref.name}` has no action" }
//                    val value =
//                        ref.action.action(newState.flatMap { it.result as? List<*> ?: listOfNotNull(it.result) })
//                    newState.last().copy(result = value)
//                }
//
//                is AST.FunctionDecl -> {
//                    val newState = params.runningFold(state) { acc, next -> next.eval(acc) }
//                    val stateWithParams = state
//                        .copy(functionParameters = newState.last().functionParameters + ref.params.zip(newState.flatMap {
//                            it.result as? List<*> ?: listOfNotNull(it.result)
//                        }).toMap())
//                    ref.rest.eval(stateWithParams).copy(functionParameters = newState.last().functionParameters)
//                }
//
//                is AST.InferredParameter -> TODO()
//                is AST.InitializedVar -> TODO()
//            }
//        }
//
//        is AST.ResolvedReference -> when (val r = referral) {
//            is AST.AbstractFunctionDecl -> TODO()
//            is AST.ForeignFunctionDecl -> TODO()
//            is AST.FunctionDecl -> TODO()
//            is AST.InitializedVar -> state.copy(result = state.variables.getValue(this))
//            is AST.InferredParameter -> state.copy(result = state.functionParameters.getValue(r))
//        }
//
//        is AST.Reference -> throw IllegalStateException(name)
//        is AST.TypeReference -> TODO()
//        is AST.LambdaExpr -> TODO()
//    }

    override fun initialState(input: List<String>) = ProgramState(input, 0, emptyMap(), null, emptyMap(), false)
    override fun join(state: ProgramState, program: AST.Node): ProgramState {
        TODO("Not yet implemented")
    }

//    override fun join(state: ProgramState, program: AST.Node): ProgramState = when (program) {
//        is AST.Expression -> program.eval(state)
//        is AST.AbstractFunctionDecl -> state
//        is AST.ForeignFunctionDecl -> state
//        is AST.FunctionDecl -> {
//            if (program.name == "main") {
//                program.rest.eval(state).let {
//                    it.copy(output = (it.result as? BigInteger)?.toInt() ?: 0, mainFound = true)
//                }
//            } else state
//        }
//
//        is AST.InferredParameter -> state
//        is AST.InitializedVar -> {
//            val expr = program.expr.eval(state)
//            expr.copy(variables = expr.variables + (program.reference as AST.ResolvedReference to expr.result!!))
//        }
//
//        is AST.Parameter -> state
//        is AST.EnumDecl.Entry -> state
//        is AST.ForeignStructDecl -> state
//        is AST.ModuleDecl -> program.rest.fold(state, ::join)
//        is AST.EnumDecl -> state
//        is AST.StructDecl -> program.rest.fold(state, ::join)
//        is AST.TraitDecl -> program.rest.fold(state, ::join)
//        is AST.Assignment -> when (val left = program.left) {
//            is AST.FunctionCall -> TODO()
//            is AST.ResolvedReference -> {
//                val expr = program.right.eval(state)
//                expr.copy(variables = expr.variables + (left to expr.result!!))
//            }
//
//            is AST.Reference -> throw IllegalStateException()
//            is AST.TypeReference -> state
//        }
//
//        is AST.IfExpr.ElifExpr -> state
//        is AST.IfExpr.ElseExpr -> state
//        is AST.FileDecl -> program.modules.fold(state, ::join)
//        is AST.TupleType -> state
//        is AST.Type -> state
//        is AST.UnionType -> state
//        is AST.WhileExpr -> {
//            var s = state
//            val list = buildList {
//                while (program.condition.eval(s).result as Boolean) {
//                    s = program.body.eval(s)
//                    add(s.result)
//                }
//            }
//            s.copy(result = list)
//        }
//        is AST.Cell<*> -> join(state, program.value)
//    }
}