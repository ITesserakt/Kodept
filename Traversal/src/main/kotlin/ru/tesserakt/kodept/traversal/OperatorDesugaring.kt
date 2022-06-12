package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.accessRLT
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

object OperatorDesugaring : Transformer<AST.BinaryOperator>() {
    override val type: KClass<AST.BinaryOperator> = AST.BinaryOperator::class

    private fun AST.BinaryOperator.replaceWith(functionName: String): AST.Dereference =
        with(accessRLT<RLT.BinaryOperation>()!!.op) {
            AST.Dereference(
                left,
                AST.FunctionCall(AST.Reference(functionName).withRLT(), listOf(right)).withRLT()
            ).withRLT()
        }

    private fun AST.Expression.callUsing(name: String, value: AST.Expression) = with(rlt) {
        AST.Dereference(
            this@callUsing,
            AST.FunctionCall(AST.Reference(name).withRLT(), listOf(value)).withRLT()
        ).withRLT()
    }

    private fun RLT.Symbol.generateOrdering(entry: String) = AST.TypeReference(
        AST.Type(entry).withRLT(),
        AST.ResolutionContext(true, listOf("Prelude", "Ordering").map(AST::Type))
    ).withRLT()

    private fun AST.Comparison.comparisonExpand() = when (kind) {
        AST.Comparison.Kind.Equal -> replaceWith("eq")
        AST.Comparison.Kind.NonEqual -> replaceWith("neq")
        AST.Comparison.Kind.Complex -> replaceWith("compare")
        else -> replaceWith("compare").callUsing(
            "eq",
            accessRLT<RLT.BinaryOperation>()!!.op.generateOrdering(kind.name)
        )
    }

    context(ReportCollector, Filepath) override fun transform(node: AST.BinaryOperator): EagerEffect<UnrecoverableError, out AST.Node> =
        eagerEffect {
            when (node) {
                is AST.Binary -> when (node.kind) {
                    AST.Binary.Kind.And -> node.replaceWith("bitAnd")
                    AST.Binary.Kind.Or -> node.replaceWith("bitOr")
                    AST.Binary.Kind.Xor -> node.replaceWith("bitXor")
                }

                is AST.Comparison -> node.comparisonExpand()
                is AST.Dereference -> node
                is AST.Elvis -> node
                is AST.Logical -> when (node.kind) {
                    AST.Logical.Kind.Conjunction -> node.replaceWith("and")
                    AST.Logical.Kind.Disjunction -> node.replaceWith("or")
                }

                is AST.Mathematical -> when (node.kind) {
                    AST.Mathematical.Kind.Add -> node.replaceWith("plus")
                    AST.Mathematical.Kind.Sub -> node.replaceWith("minus")
                    AST.Mathematical.Kind.Mul -> node.replaceWith("times")
                    AST.Mathematical.Kind.Div -> node.replaceWith("divide")
                    AST.Mathematical.Kind.Mod -> node.replaceWith("modulo")
                    AST.Mathematical.Kind.Pow -> node.replaceWith("power")
                }
            }
        }
}