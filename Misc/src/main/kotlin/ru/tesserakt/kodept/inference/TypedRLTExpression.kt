package ru.tesserakt.kodept.inference

import arrow.core.Either
import arrow.core.NonEmptyList
import ru.tesserakt.kodept.core.RLT

sealed interface TypedRLTExpression {
    val type: KnownType

    data class BinaryOperation(
        val left: TypedRLTExpression,
        val right: TypedRLTExpression,
        val op: RLT.Symbol,
        override val type: KnownType,
    ) : TypedRLTExpression

    data class UnaryOperation(
        val expr: TypedRLTExpression,
        val op: RLT.Symbol,
        override val type: KnownType,
    ) : TypedRLTExpression

    data class Block(
        val subexpressions: NonEmptyList<Either<RLT.StatementNode, TypedRLTExpression>>,
        override val type: KnownType,
    ) : TypedRLTExpression

    data class TupleLiteral(
        val items: List<TypedRLTExpression>,
        override val type: KnownType,
    ) : TypedRLTExpression {
        companion object {
            val unit = TupleLiteral(emptyList(), KnownType.unit)
        }
    }

    data class If(
        val condition: TypedRLTExpression,
        val body: TypedRLTExpression,
        val elifs: List<Elif>,
        val el: TypedRLTExpression,
        override val type: KnownType,
    ) : TypedRLTExpression {
        data class Elif(val condition: TypedRLTExpression, val body: TypedRLTExpression, override val type: KnownType) :
            TypedRLTExpression
    }

    data class While(
        val condition: TypedRLTExpression,
        val body: TypedRLTExpression,
        override val type: KnownType,
    ) : TypedRLTExpression

    object BottomTypeLiteral : TypedRLTExpression {
        override val type = KnownType.BottomType
    }

    data class FloatingLiteral(val number: RLT.Literal.Floating, override val type: KnownType) : TypedRLTExpression

    data class NumberLiteral(val number: RLT.Literal.Number, override val type: KnownType) : TypedRLTExpression

    data class TextLiteral(val textLiteral: RLT.Literal.Text, override val type: KnownType) : TypedRLTExpression

    data class Application(
        val reciever: TypedRLTExpression, val params: List<TypedRLTExpression>,
        override val type: KnownType,
    ) : TypedRLTExpression

    data class Reference(
        val ref: RLT.UserSymbol,
        override val type: KnownType,
    ) : TypedRLTExpression
}