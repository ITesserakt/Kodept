package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

object BinaryOperatorDesugaring : Transformer<AST.BinaryOperator>() {
    override val type: KClass<AST.BinaryOperator> = AST.BinaryOperator::class

    private fun AST.BinaryOperator.replaceWith(functionName: String, traitName: String): AST.Dereference =
        with(accessRLT<RLT.BinaryOperation>()?.op ?: accessRLT<RLT.CompoundAssignment>()!!.compoundOperator) {
            AST.Dereference(
                (left as? AST.BinaryOperator)?.expand() ?: left,
                AST.FunctionCall(
                    AST.Reference(
                        functionName,
                        AST.ResolutionContext(true, listOf("Prelude", traitName).map(AST::Type))
                    ).withRLT(),
                    listOf((right as? AST.BinaryOperator)?.expand() ?: right)
                ).withRLT()
            ).withRLT()
        }

    private fun AST.BinaryOperator.expand() = when (this) {
        is AST.Binary -> when (kind) {
            AST.Binary.Kind.And -> replaceWith("bitAnd", "Integral")
            AST.Binary.Kind.Or -> replaceWith("bitOr", "Integral")
            AST.Binary.Kind.Xor -> replaceWith("bitXor", "Integral")
        }

        is AST.Comparison -> when (kind) {
            AST.Comparison.Kind.Equal -> replaceWith("eq", "Eq")
            AST.Comparison.Kind.NonEqual -> replaceWith("neq", "Eq")
            AST.Comparison.Kind.Complex -> replaceWith("compare", "Ord")
            AST.Comparison.Kind.Less -> replaceWith("less", "Ord")
            AST.Comparison.Kind.LessEqual -> replaceWith("lessEq", "Ord")
            AST.Comparison.Kind.GreaterEqual -> replaceWith("greaterEq", "Ord")
            AST.Comparison.Kind.Greater -> replaceWith("greater", "Ord")
        }
        is AST.Dereference -> this
        is AST.Elvis -> this
        is AST.Logical -> when (kind) {
            AST.Logical.Kind.Conjunction -> replaceWith("and", "BoolLike")
            AST.Logical.Kind.Disjunction -> replaceWith("or", "BoolLike")
        }

        is AST.Mathematical -> when (kind) {
            AST.Mathematical.Kind.Add -> replaceWith("plus", "Num")
            AST.Mathematical.Kind.Sub -> replaceWith("minus", "Num")
            AST.Mathematical.Kind.Mul -> replaceWith("times", "Num")
            AST.Mathematical.Kind.Div -> replaceWith("divide", "Fractional")
            AST.Mathematical.Kind.Mod -> replaceWith("modulo", "Integral")
            AST.Mathematical.Kind.Pow -> replaceWith("power", "Integral")
        }
    }

    context(ReportCollector, Filepath) override fun transform(node: AST.BinaryOperator): EagerEffect<UnrecoverableError, out AST.Node> {
        return eagerEffect { node.expand() }
    }

    val contract = Contract<AST.BinaryOperator, Depended> {
        require(this !is AST.Dereference)
        "binary operator $this should not be in AST. Try adding $BinaryOperatorDesugaring as a dependency in $it"
    }
}

object UnaryOperatorDesugaring : Transformer<AST.UnaryOperator>() {
    override val type: KClass<AST.UnaryOperator> = AST.UnaryOperator::class

    private fun AST.UnaryOperator.replaceWith(functionName: String) =
        with(accessRLT<RLT.UnaryOperation>()!!.op) {
            AST.Dereference(
                expr,
                AST.FunctionCall(AST.Reference(functionName, null).withRLT(), emptyList()).withRLT()
            ).withRLT()
        }

    context(ReportCollector, Filepath) override fun transform(node: AST.UnaryOperator): EagerEffect<UnrecoverableError, out AST.Node> =
        eagerEffect {
            when (node) {
                is AST.Absolution -> node.replaceWith("unaryPlus")
                is AST.BitInversion -> node.replaceWith("bitNot")
                is AST.Inversion -> node.replaceWith("not")
                is AST.Negation -> node.replaceWith("unaryMinus")
            }
        }

    val contract = Contract<AST.UnaryOperator, Depended> {
        "unary operator $this should not be in AST. Try adding $UnaryOperatorDesugaring as a dependency in $it"
    }
}