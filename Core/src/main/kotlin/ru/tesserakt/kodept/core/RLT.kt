package ru.tesserakt.kodept.core

import arrow.core.Eval
import arrow.core.NonEmptyList

/**
 * Raw lexem tree - it has all information about tokenized lexems
 */
data class RLT(val root: File) {
    sealed interface Node {
        val description: String

        val position: CodePoint
        val text: Eval<String>
    }

    data class Keyword(override val text: Eval<String>, override val position: CodePoint) : Node {
        override val description = text.value()
    }

    sealed class UserSymbol : Node {
        class Identifier(override val text: Eval<String>, override val position: CodePoint) : UserSymbol() {
            override val description = "identifier"

            override fun equals(other: Any?) = other is Identifier && text.value() == other.text.value()
            override fun hashCode(): Int = text.value().hashCode()
        }

        class Type(override val text: Eval<String>, override val position: CodePoint) : UserSymbol(), Bind {
            override val description = "type"

            override fun equals(other: Any?) = other is Type && text.value() == other.text.value()
            override fun hashCode(): Int = text.value().hashCode()
        }
    }

    data class Symbol(override val text: Eval<String>, override val position: CodePoint, val type: String) : Node {
        override val description = text.value()
    }

    class ParameterTuple(
        val lparen: Symbol,
        val params: List<Parameter>,
        val rparen: Symbol,
    ) : ExpressionNode, Node by lparen

    open class MaybeTypedParameterTuple(
        val lparen: Symbol,
        open val params: List<MaybeTypedParameter>,
        val rparen: Symbol,
    )

    class TypedParameterTuple(lparen: Symbol, override val params: List<TypedParameter>, rparen: Symbol) :
        MaybeTypedParameterTuple(lparen, params, rparen)

    class Parameter(val id: ExpressionNode) : ExpressionNode, Node by id
    open class MaybeTypedParameter(override val id: UserSymbol.Identifier, open val type: TypeNode?) : Bind,
        Named, Node by id

    class TypedParameter(id: UserSymbol.Identifier, override val type: TypeNode) : MaybeTypedParameter(id, type)

    data class File(val moduleList: NonEmptyList<Module>) : Node by moduleList.head {
        override val description = "file"
    }

    sealed interface TopLevelNode : Node
    sealed interface ObjectLevelNode : Node
    sealed interface StructLevelNode : ObjectLevelNode
    sealed interface TraitLevelNode : ObjectLevelNode
    sealed interface BlockLevelNode : Node, Bind
    sealed interface ExpressionNode : BlockLevelNode
    sealed interface StatementNode : BlockLevelNode
    sealed interface Lvalue : Node
    sealed interface TermNode : ExpressionNode, Lvalue
    sealed interface TypeNode : Node
    sealed interface Bind : Node
    sealed interface Scoping : Bind
    sealed interface Named : Node {
        val id: UserSymbol
    }

    sealed class Module(val keyword: Keyword, override val id: UserSymbol.Type, val rest: List<TopLevelNode>) : Scoping,
        Named, Node by keyword {
        class Global(keyword: Keyword, id: UserSymbol.Type, flow: Symbol, rest: List<TopLevelNode>) :
            Module(keyword, id, rest)

        class Ordinary(
            keyword: Keyword,
            id: UserSymbol.Type,
            val lbrace: Symbol,
            rest: List<TopLevelNode>,
            val rbrace: Symbol,
        ) : Module(keyword, id, rest)
    }

    data class ForeignType(
        val fKeyword: Keyword,
        val tKeyword: Keyword,
        val id: UserSymbol.Type,
        val flow: Symbol,
        val type: Literal.Text,
    ) : TopLevelNode, Node by id

    data class Struct(
        val keyword: Keyword,
        override val id: UserSymbol.Type,
        val lparen: Symbol?,
        val varsToAlloc: List<TypedParameter>,
        val rparen: Symbol?,
        val lbrace: Symbol?,
        val rest: List<StructLevelNode>,
        val rbrace: Symbol?,
    ) : TopLevelNode, Scoping, Named, Node by keyword

    data class Trait(
        val keyword: Keyword,
        override val id: UserSymbol.Type,
        val lbrace: Symbol?,
        val rest: List<TraitLevelNode>,
        val rbrace: Symbol?,
    ) : TopLevelNode, Scoping, Named, Node by keyword

    sealed class Enum(
        val keyword: Keyword,
        override val id: UserSymbol.Type,
        val lbrace: Symbol?,
        val rest: NonEmptyList<UserSymbol.Type>,
        val rbrace: Symbol?,
    ) : TopLevelNode, Scoping, Named, Node by keyword {
        class Stack(
            keyword: Keyword,
            id: UserSymbol.Type,
            lbrace: Symbol?,
            rest: NonEmptyList<UserSymbol.Type>,
            rbrace: Symbol?,
        ) : Enum(keyword, id, lbrace, rest, rbrace)

        class Heap(
            keyword: Keyword,
            id: UserSymbol.Type,
            lbrace: Symbol?,
            rest: NonEmptyList<UserSymbol.Type>,
            rbrace: Symbol?,
        ) : Enum(keyword, id, lbrace, rest, rbrace)
    }

    sealed interface Body : ExpressionNode, Scoping {
        data class Expression(val flow: Symbol, val expression: ExpressionNode) : Body, ExpressionNode by expression
        data class Block(val lbrace: Symbol, val block: List<BlockLevelNode>, val rbrace: Symbol) : Body, Node by lbrace
    }

    sealed class Function(
        val keyword: Keyword,
        override val id: UserSymbol.Identifier,
        open val params: List<MaybeTypedParameterTuple>,
        val colon: Symbol?,
        open val returnType: TypeNode?,
    ) : Named, Node by keyword {
        class Abstract(
            keyword: Keyword,
            id: UserSymbol.Identifier,
            override val params: List<TypedParameterTuple>,
            colon: Symbol?,
            returnType: TypeNode?,
        ) : Function(keyword, id, params, colon, returnType), Bind, TraitLevelNode

        class Bodied(
            keyword: Keyword,
            id: UserSymbol.Identifier,
            params: List<MaybeTypedParameterTuple>,
            colon: Symbol?,
            returnType: TypeNode?,
            val body: Body,
        ) : Function(keyword, id, params, colon, returnType), TopLevelNode, StructLevelNode, StatementNode, Scoping,
            TraitLevelNode

        class Foreign(
            keyword: Keyword,
            id: UserSymbol.Identifier,
            override val params: List<TypedParameterTuple>,
            colon: Symbol?,
            override val returnType: Reference?,
            val descriptor: Literal.Text,
        ) : Function(keyword, id, params, colon, returnType), TopLevelNode
    }

    open class Reference(val ref: UserSymbol) : TermNode, TypeNode, Node by ref

    data class Application(val expr: Reference, val params: List<ParameterTuple>) : TermNode, Node by expr

    sealed interface Context {
        val global: Boolean

        data class Global(val colon: Symbol) : Context {
            override val global = true
        }

        object Local : Context {
            override val global = false
        }

        data class Inner(val type: Reference, val parent: Context) : Context {
            override val global = parent.global
        }
    }

    class ContextualReference(val context: Context, reference: Reference) : Reference(reference.ref)

    sealed class Variable(
        val keyword: Keyword,
        override val id: UserSymbol.Identifier,
        val colon: Symbol?,
        val type: TypeNode?,
    ) : StatementNode, Lvalue, Named, Node by keyword {
        class Immutable(keyword: Keyword, id: UserSymbol.Identifier, colon: Symbol?, type: TypeNode?) :
            Variable(keyword, id, colon, type)

        class Mutable(keyword: Keyword, id: UserSymbol.Identifier, colon: Symbol?, type: TypeNode?) :
            Variable(keyword, id, colon, type)
    }

    open class Assignment(open val lvalue: Lvalue, val equals: Symbol, val expression: ExpressionNode) : StatementNode,
        Node by equals

    class InitializedAssignment(override val lvalue: Variable, equals: Symbol, expression: ExpressionNode) :
        Assignment(lvalue, equals, expression), Named {
        override val id: UserSymbol = lvalue.id
    }

    class CompoundAssignment(lvalue: Lvalue, val compoundOperator: Symbol, expression: ExpressionNode) :
        Assignment(lvalue, compoundOperator, expression)

    data class BinaryOperation(val left: ExpressionNode, val op: Symbol, val right: ExpressionNode) : ExpressionNode,
        Node by op

    data class UnaryOperation(val expression: ExpressionNode, val op: Symbol) : ExpressionNode, Node by op

    sealed class Literal(override val text: Eval<String>, override val position: CodePoint) : ExpressionNode {
        override val description get() = text.value()

        class Number(text: Eval<String>, position: CodePoint) : Literal(text, position)
        class Floating(text: Eval<String>, position: CodePoint) : Literal(text, position)
        class Text(text: Eval<String>, position: CodePoint) : Literal(text, position) {
            fun isChar() = text.value().first() == '\''
            fun isString() = text.value().first() == '"'
        }

        data class Tuple(val lparen: Symbol, val expressions: List<ExpressionNode>, val rparen: Symbol) :
            Literal(lparen.text, lparen.position)
    }

    data class If(
        val keyword: Keyword,
        val condition: ExpressionNode,
        val body: Body,
        val elifs: List<Elif>,
        val el: Else?,
    ) : ExpressionNode, Node by keyword {
        data class Elif(val keyword: Keyword, val condition: ExpressionNode, val body: Body) : Node by keyword
        data class Else(val keyword: Keyword, val body: Body) : Node by keyword
    }

    data class While(
        val keyword: Keyword,
        val condition: ExpressionNode,
        val body: Body,
    ) : ExpressionNode, Node by keyword

    data class TupleType(val lparen: Symbol, val types: List<TypeNode>, val rparen: Symbol) : TypeNode, Node by lparen

    data class UnionType(val lparen: Symbol, val types: NonEmptyList<TypeNode>, val rparen: Symbol) : TypeNode,
        Node by lparen
}