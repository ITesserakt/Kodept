@file:Suppress("unused", "DuplicatedCode")

package ru.tesserakt.kodept.core

import arrow.core.NonEmptyList
import arrow.core.prependTo
import ru.tesserakt.kodept.core.Tree.SearchMode
import java.math.BigDecimal
import java.math.BigInteger
import kotlin.reflect.KMutableProperty0

data class AST(val root: Node, val filename: Filename) {
    init {
        walkThrough { node -> node.children().forEach { it.parent = node } }.forEach { _ -> }
    }

    fun <T> walkThrough(mode: SearchMode = SearchMode.LevelOrder, f: (Node) -> T) = root.walkTopDown(mode, f)

    fun flatten(mode: SearchMode = SearchMode.LevelOrder) = root.gatherChildren(mode)

    sealed interface Node : Tree<Node> {
        override var parent: Node?
        val rlt: RLT.Node

        fun <A : Node?> replaceChild(old: A, new: A): Boolean
    }

    sealed interface Named : Node {
        val name: String
    }

    sealed interface TopLevel : Named
    sealed interface ObjectLevel : Named
    sealed interface StructLevel : ObjectLevel
    sealed interface TraitLevel : ObjectLevel
    sealed interface EnumLevel : ObjectLevel
    sealed interface BlockLevel : Node
    sealed interface Expression : BlockLevel
    sealed interface Statement : BlockLevel
    sealed interface Literal : Expression
    sealed interface Lvalue : Expression
    sealed interface Referable : Statement, Named

    sealed class NodeBase : Node {
        final override var parent: Node? = null
        internal lateinit var _rlt: RLT.Node

        protected inline fun <reified T : Node> MutableList<T>.replace(old: Node?, new: Node?) =
            old is T && remove(old) && new is T && add(new)

        protected inline fun <reified T : Node?> KMutableProperty0<T>.replace(old: Node?, new: Node?) =
            new is T && get() == old && true.apply { set(new) }
    }

    sealed class Leaf : Node {
        final override var parent: Node? = null
        final override fun children() = emptyList<Node>()
        final override fun <A : Node?> replaceChild(old: A, new: A): Boolean = false
        internal lateinit var _rlt: RLT.Node
    }

    class Stub(private val prototype: Node) : Leaf(), Literal, Statement {
        override val rlt: RLT.Node get() = prototype.rlt
    }

    class Parameter(override val name: String, type: TypeExpression) : NodeBase(), Referable {
        var type = type
            private set

        override fun children() = listOf(type)

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::type.replace(old, new)

        override fun toString(): String {
            return "Parameter(name='$name', type=$type)"
        }

        fun copy(name: String = this.name, type: TypeExpression = this.type) = Parameter(name, type)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as Parameter

            if (name != other.name) return false
            if (type != other.type) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + type.hashCode()
            return result
        }

        override val rlt get() = _rlt as RLT.TypedParameter
    }

    class InferredParameter(override val name: String, type: TypeExpression?) : NodeBase(), Referable {
        var type = type
            private set

        override fun children() = listOfNotNull(type)

        fun copy(name: String = this.name, type: TypeExpression? = this.type) = InferredParameter(name, type)

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::type.replace(old, new)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as InferredParameter

            if (name != other.name) return false
            if (type != other.type) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + (type?.hashCode() ?: 0)
            return result
        }

        override fun toString(): String {
            return "InferredParameter(name='$name', type=$type)"
        }

        override val rlt: RLT.MaybeTypedParameter get() = _rlt as RLT.MaybeTypedParameter
    }

    data class FileDecl(private val _modules: MutableList<ModuleDecl>) : NodeBase() {
        val modules get() = NonEmptyList.fromListUnsafe(_modules)

        override fun children() = modules

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _modules.replace(old, new)

        constructor(modules: NonEmptyList<ModuleDecl>) : this(modules.toMutableList())

        override val rlt: RLT.File get() = _rlt as RLT.File
    }

    data class ModuleDecl(override val name: String, val global: Boolean, private val _rest: MutableList<TopLevel>) :
        NodeBase(), Named {
        val rest get() = _rest.toList()

        override fun children() = rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _rest.replace(old, new)

        constructor(name: String, global: Boolean, rest: Iterable<TopLevel>) : this(name, global, rest.toMutableList())

        override val rlt: RLT.Module get() = _rlt as RLT.Module
    }

    data class StructDecl(
        override val name: String,
        private val _alloc: MutableList<Parameter>,
        private val _rest: MutableList<StructLevel>,
    ) :
        NodeBase(), TopLevel {
        val alloc get() = _alloc.toList()
        val rest get() = _rest.toList()

        override fun children() = alloc + rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            _alloc.replace(old, new) || _rest.replace(old, new)

        constructor(name: String, alloc: Iterable<Parameter>, rest: Iterable<StructLevel>) : this(
            name, alloc.toMutableList(), rest.toMutableList()
        )

        override val rlt: RLT.Struct get() = _rlt as RLT.Struct
    }

    data class EnumDecl(
        override val name: String,
        val stackBased: Boolean,
        private val _enumEntries: MutableList<EnumLevel>,
    ) :
        NodeBase(), TopLevel {
        val enumEntries get() = _enumEntries.toList()
        override fun children() = enumEntries

        data class Entry(override val name: String) : Leaf(), EnumLevel {
            override val rlt: RLT.UserSymbol.Type get() = _rlt as RLT.UserSymbol.Type
        }

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _enumEntries.replace(old, new)

        constructor(name: String, stackBased: Boolean, enumEntries: Iterable<EnumLevel>) : this(
            name, stackBased, enumEntries.toMutableList()
        )

        override val rlt: RLT.Enum get() = _rlt as RLT.Enum
    }

    data class TraitDecl(override val name: String, private val _rest: MutableList<TraitLevel>) : NodeBase(), TopLevel {
        val rest get() = _rest.toList()
        override fun children() = rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _rest.replace(old, new)

        constructor(name: String, rest: Iterable<TraitLevel>) : this(name, rest.toMutableList())

        override val rlt: RLT.Trait get() = _rlt as RLT.Trait
    }

    class AbstractFunctionDecl(
        override val name: String,
        private val _params: MutableList<Parameter>, returns: TypeExpression?,
    ) : NodeBase(), TraitLevel {
        var returns = returns
            private set
        val params get() = _params.toList()
        override fun children() = params + listOfNotNull(returns)

        override fun <A : Node?> replaceChild(old: A, new: A) = ::returns.replace(old, new) || _params.replace(old, new)

        fun copy(
            name: String = this.name,
            params: Iterable<Parameter> = this._params,
            returns: TypeExpression? = this.returns,
        ) = AbstractFunctionDecl(name, params, returns)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as AbstractFunctionDecl

            if (name != other.name) return false
            if (_params != other._params) return false
            if (returns != other.returns) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + _params.hashCode()
            result = 31 * result + (returns?.hashCode() ?: 0)
            return result
        }

        override fun toString(): String {
            return "AbstractFunctionDecl(name='$name', params=$_params, returns=$returns)"
        }

        constructor(name: String, params: Iterable<Parameter>, returns: TypeExpression?) : this(
            name, params.toMutableList(), returns
        )

        override val rlt: RLT.Function.Abstract get() = _rlt as RLT.Function.Abstract
    }

    class FunctionDecl(
        override val name: String,
        private val _params: MutableList<InferredParameter>, returns: TypeExpression?, rest: Expression,
    ) : NodeBase(), TopLevel, StructLevel, TraitLevel, Referable {
        var returns = returns
            private set
        var rest = rest
            private set
        val params get() = _params.toList()
        override fun children() = params + listOfNotNull(returns) + listOf(rest)

        override fun <A : Node?> replaceChild(old: A, new: A) =
            ::returns.replace(old, new) || ::rest.replace(old, new) || _params.replace(old, new)

        fun copy(
            name: String = this.name,
            params: Iterable<InferredParameter> = this._params,
            returns: TypeExpression? = this.returns,
            rest: Expression = this.rest,
        ) = FunctionDecl(name, params, returns, rest)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as FunctionDecl

            if (name != other.name) return false
            if (_params != other._params) return false
            if (returns != other.returns) return false
            if (rest != other.rest) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + _params.hashCode()
            result = 31 * result + (returns?.hashCode() ?: 0)
            result = 31 * result + rest.hashCode()
            return result
        }

        override fun toString(): String {
            return "FunctionDecl(name='$name', params=$_params, returns=$returns, rest=$rest)"
        }

        constructor(
            name: String,
            params: Iterable<InferredParameter>,
            returns: TypeExpression?,
            rest: Expression,
        ) : this(name, params.toMutableList(), returns, rest)

        override val rlt: RLT.Function.Bodied get() = _rlt as RLT.Function.Bodied
    }

    class InitializedVar(reference: Reference, val mutable: Boolean, type: TypeExpression?, expr: Expression) :
        NodeBase(), Referable {
        var reference: Reference = reference
            private set
        var type: TypeExpression? = type
            private set
        var expr: Expression = expr
            private set
        override val name: String = reference.name

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            ::reference.replace(old, new) || ::type.replace(old, new) || ::expr.replace(old, new)

        override fun children() = listOf(reference) + listOfNotNull(type) + listOf(expr)

        fun copy(
            reference: Reference = this.reference,
            mutable: Boolean = this.mutable,
            type: TypeExpression? = this.type,
            expr: Expression = this.expr,
        ) = InitializedVar(reference, mutable, type, expr)

        override fun toString(): String {
            return "InitializedVar(reference=$reference, mutable=$mutable, type=$type expr=$expr)"
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as InitializedVar

            if (mutable != other.mutable) return false
            if (reference != other.reference) return false
            if (type != other.type) return false
            if (expr != other.expr) return false
            if (name != other.name) return false

            return true
        }

        override fun hashCode(): Int {
            var result = mutable.hashCode()
            result = 31 * result + reference.hashCode()
            result = 31 * result + (type?.hashCode() ?: 0)
            result = 31 * result + expr.hashCode()
            result = 31 * result + name.hashCode()
            return result
        }

        override val rlt: RLT.InitializedAssignment get() = _rlt as RLT.InitializedAssignment
    }

    data class DecimalLiteral(val value: BigInteger) : Leaf(), Literal {
        override val rlt get() = _rlt as RLT.Literal.Floating
    }

    data class BinaryLiteral(val value: BigInteger) : Leaf(), Literal {
        override val rlt get() = _rlt as RLT.Literal.Number
    }

    data class OctalLiteral(val value: BigInteger) : Leaf(), Literal {
        override val rlt get() = _rlt as RLT.Literal.Number
    }

    data class HexLiteral(val value: BigInteger) : Leaf(), Literal {
        override val rlt get() = _rlt as RLT.Literal.Number
    }

    data class CharLiteral(val value: Char) : Leaf(), Literal {
        override val rlt get() = _rlt as RLT.Literal.Text
    }

    data class StringLiteral(val value: String) : Leaf(), Literal {
        override val rlt get() = _rlt as RLT.Literal.Text
    }

    data class FloatingLiteral(val value: BigDecimal) : Leaf(), Literal {
        override val rlt get() = _rlt as RLT.Literal.Floating
    }

    data class TupleLiteral(private val _items: MutableList<Node>) : NodeBase(), Literal {
        val items get() = _items.toList()

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _items.replace(old, new)

        override fun children() = items

        @Suppress("unused")
        val arity = items.size

        companion object {
            val unit = TupleLiteral(mutableListOf())
        }

        constructor(items: Iterable<Node>) : this(items.toMutableList())

        override val rlt get() = _rlt as RLT.ExpressionNode
    }

    sealed class BinaryOperator : NodeBase(), Expression {
        abstract var left: Expression
            protected set
        abstract var right: Expression
            protected set

        override fun children(): List<Node> = listOf(left, right)

        override fun <A : Node?> replaceChild(old: A, new: A) =
            new is Expression && (left == old && true.apply { left = new } ||
                    right == old && true.apply { right = new })

        override val rlt: RLT.BinaryOperation get() = _rlt as RLT.BinaryOperation
    }

    data class Mathematical(override var left: Expression, override var right: Expression, val kind: Kind) :
        BinaryOperator() {
        enum class Kind { Add, Sub, Mul, Div, Mod, Pow }
    }

    data class Logical(override var left: Expression, override var right: Expression, val kind: Kind) :
        BinaryOperator() {
        enum class Kind { Conjunction, Disjunction }
    }

    data class Comparison(override var left: Expression, override var right: Expression, val kind: Kind) :
        BinaryOperator() {
        enum class Kind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }
    }

    data class Binary(override var left: Expression, override var right: Expression, val kind: Kind) :
        BinaryOperator() {
        enum class Kind { And, Or, Xor }
    }

    sealed class UnaryOperator : NodeBase(), Expression {
        abstract var expr: Expression
            protected set

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            (::expr as KMutableProperty0<Expression>).replace(old, new)

        override fun children(): List<Node> = listOf(expr)

        override val rlt get() = _rlt as RLT.UnaryOperation
    }

    data class Negation(override var expr: Expression) : UnaryOperator()
    data class Inversion(override var expr: Expression) : UnaryOperator()
    data class BitInversion(override var expr: Expression) : UnaryOperator()
    data class Absolution(override var expr: Expression) : UnaryOperator()
    data class Elvis(override var left: Expression, override var right: Expression) : BinaryOperator()
    class Assignment(left: Lvalue, right: Expression) : NodeBase(), Expression {
        var left = left
            private set
        var right = right
            private set

        override fun children() = listOf(left, right)
        override fun <A : Node?> replaceChild(old: A, new: A) = ::left.replace(old, new) || ::right.replace(old, new)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as Assignment

            if (left != other.left) return false
            if (right != other.right) return false

            return true
        }

        override fun hashCode(): Int {
            var result = left.hashCode()
            result = 31 * result + right.hashCode()
            return result
        }

        override fun toString(): String {
            return "Assignment(left=$left, right=$right)"
        }

        override val rlt get() = _rlt as RLT.Assignment
    }

    data class ResolutionContext(val fromRoot: Boolean, val chain: List<Type>)
    open class Reference(override val name: String, val resolutionContext: ResolutionContext? = null) :
        Leaf(), Lvalue, Named {
        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as Reference

            if (name != other.name) return false
            if (resolutionContext != other.resolutionContext) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + (resolutionContext?.hashCode() ?: 0)
            return result
        }

        override fun toString(): String {
            return "Reference(name='$name', resolutionContext=$resolutionContext)"
        }

        fun copy(name: String = this.name, resolutionContext: ResolutionContext? = this.resolutionContext) =
            Reference(name, resolutionContext)

        override val rlt get() = _rlt as RLT.Reference
    }

    class ResolvedReference(name: String, val referral: Referable, resolutionContext: ResolutionContext? = null) :
        Reference(name, resolutionContext) {
        constructor(ref: Reference, referral: Referable) : this(ref.name, referral, ref.resolutionContext) {
            _rlt = ref.rlt
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false
            if (!super.equals(other)) return false

            other as ResolvedReference

            if (referral != other.referral) return false

            return true
        }

        override fun hashCode(): Int {
            var result = super.hashCode()
            result = 31 * result + referral.hashCode()
            return result
        }

        override fun toString(): String {
            return "ResolvedReference(name='$name', referral=$referral, resolutionContext=$resolutionContext)"
        }
    }

    class TypeReference(type: TypeExpression, val resolutionContext: ResolutionContext? = null) : NodeBase(), Lvalue {
        var type = type
            private set

        override fun children() = listOf(type)
        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::type.replace(old, new)

        fun copy(type: TypeExpression = this.type, resolutionContext: ResolutionContext? = this.resolutionContext) =
            TypeReference(type, resolutionContext)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as TypeReference

            if (resolutionContext != other.resolutionContext) return false
            if (type != other.type) return false

            return true
        }

        override fun hashCode(): Int {
            var result = resolutionContext?.hashCode() ?: 0
            result = 31 * result + type.hashCode()
            return result
        }

        override fun toString(): String {
            return "TypeReference(resolutionContext=$resolutionContext, type=$type)"
        }

        override val rlt get() = _rlt as RLT.UserSymbol.Type
    }

    data class FunctionCall(
        val reference: Node,
        private val _params: MutableList<Node>,
        val resolutionContext: ResolutionContext? = null,
    ) : NodeBase(), Lvalue {
        val params get() = _params.toList()
        override fun children() = reference.prependTo(params)
        override fun <A : Node?> replaceChild(old: A, new: A) = _params.replace(old, new)

        constructor(reference: Node, params: Iterable<Node>, resolutionContext: ResolutionContext? = null) : this(
            reference, params.toMutableList(), resolutionContext
        )

        override val rlt get() = _rlt as RLT.Application
    }

    data class Dereference(override var left: Expression, override var right: Expression) : BinaryOperator(), Lvalue

    data class ExpressionList(private val _expressions: MutableList<BlockLevel>) : NodeBase(), Expression {
        val expressions get() = _expressions.toList()
        override fun children() = expressions
        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _expressions.replace(old, new)

        constructor(expressions: Iterable<BlockLevel>) : this(expressions.toMutableList())

        override val rlt get() = _rlt as RLT.Body.Block
    }

    sealed class TypeExpression : NodeBase()

    data class Type(val name: String) : TypeExpression() {
        override fun <A : Node?> replaceChild(old: A, new: A) = false

        override fun children() = emptyList<Node>()

        override fun toString() = name

        override val rlt get() = _rlt as RLT.UserSymbol.Type
    }

    data class TupleType(private val _items: MutableList<TypeExpression>) : TypeExpression() {
        val items get() = _items.toList()

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _items.replace(old, new)

        override fun children() = items

        override fun toString() = items.joinToString(prefix = "(", postfix = ")")

        companion object {
            val unit = TupleType(mutableListOf())
        }

        constructor(items: Iterable<TypeExpression>) : this(items.toMutableList())

        override fun equals(other: Any?): Boolean =
            other is TupleType && _items.containsAll(other._items) && other._items.containsAll(_items)

        override fun hashCode() = _items.hashCode()

        override val rlt get() = _rlt as RLT.TupleType
    }

    data class UnionType(private val _items: MutableList<TypeExpression>) : TypeExpression() {
        val items get() = NonEmptyList.fromListUnsafe(_items)

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _items.replace(old, new)

        override fun children() = items

        override fun toString() = items.joinToString(" | ", "(", ")")

        constructor(items: NonEmptyList<TypeExpression>) : this(items.toMutableList())

        override fun equals(other: Any?): Boolean =
            other is UnionType && _items.containsAll(other._items) && other._items.containsAll(_items)

        override fun hashCode(): Int = _items.hashCode()

        override val rlt get() = _rlt as RLT.UnionType

        fun copy(_items: NonEmptyList<TypeExpression> = this.items) = UnionType(_items).also { it._rlt = rlt }
    }

    class IfExpr(
        condition: Node,
        body: Node,
        private val _elifs: MutableList<ElifExpr>,
        el: ElseExpr?,
    ) : NodeBase(), Expression {
        var condition = condition
            private set
        var body = body
            private set
        var el = el
            private set

        constructor(condition: Node, body: Node, elifs: Iterable<ElifExpr>, el: ElseExpr?) : this(
            condition, body, elifs.toMutableList(), el
        )

        fun copy(
            condition: Node = this.condition,
            body: Node = this.body,
            elifs: Iterable<ElifExpr> = this.elifs,
            el: ElseExpr? = this.el,
        ) = IfExpr(condition, body, elifs, el)

        val elifs get() = _elifs.toList()

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            ::condition.replace(old, new) || ::body.replace(old, new) || _elifs.replace(old, new) || ::el.replace(
                old, new
            )

        override fun children() = listOf(condition, body) + elifs + listOfNotNull(el)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as IfExpr

            if (_elifs != other._elifs) return false
            if (condition != other.condition) return false
            if (body != other.body) return false
            if (el != other.el) return false

            return true
        }

        override fun hashCode(): Int {
            var result = _elifs.hashCode()
            result = 31 * result + condition.hashCode()
            result = 31 * result + body.hashCode()
            result = 31 * result + (el?.hashCode() ?: 0)
            return result
        }

        override fun toString(): String {
            return "IfExpr(condition=$condition, body=$body, elifs=$_elifs, el=$el)"
        }

        override val rlt get() = _rlt as RLT.If

        class ElifExpr(condition: Node, body: Node) : NodeBase() {
            var condition = condition
                private set
            var body = body
                private set

            override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
                ::condition.replace(old, new) || ::body.replace(old, new)

            override fun children() = listOf(condition, body)

            fun copy(condition: Node = this.condition, body: Node = this.body) = ElifExpr(condition, body)

            override fun equals(other: Any?): Boolean {
                if (this === other) return true
                if (javaClass != other?.javaClass) return false

                other as ElifExpr

                if (condition != other.condition) return false
                if (body != other.body) return false

                return true
            }

            override fun hashCode(): Int {
                var result = condition.hashCode()
                result = 31 * result + body.hashCode()
                return result
            }

            override fun toString(): String {
                return "ElifExpr(condition=$condition, body=$body)"
            }

            override val rlt get() = _rlt as RLT.If.Elif
        }

        class ElseExpr(body: Node) : NodeBase() {
            var body = body
                private set

            override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::body.replace(old, new)

            override fun children() = listOf(body)

            fun copy(body: Node = this.body) = ElseExpr(body)

            override fun equals(other: Any?): Boolean {
                if (this === other) return true
                if (javaClass != other?.javaClass) return false

                other as ElseExpr

                if (body != other.body) return false

                return true
            }

            override fun hashCode(): Int {
                return body.hashCode()
            }

            override fun toString(): String {
                return "ElseExpr(body=$body)"
            }

            override val rlt get() = _rlt as RLT.If.Else
        }
    }

    class WhileExpr(condition: Node, body: Node) : NodeBase(), Expression {
        var condition = condition
            private set
        var body = body
            private set

        fun copy(condition: Node = this.condition, body: Node = this.body) = WhileExpr(condition, body)

        override fun <A : Node?> replaceChild(old: A, new: A) =
            ::condition.replace(old, new) || ::body.replace(old, new)

        override fun children() = listOf(condition, body)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as WhileExpr

            if (condition != other.condition) return false
            if (body != other.body) return false

            return true
        }

        override fun hashCode(): Int {
            var result = condition.hashCode()
            result = 31 * result + body.hashCode()
            return result
        }

        override fun toString(): String {
            return "WhileExpr(condition=$condition, body=$body)"
        }

        override val rlt get() = _rlt as RLT.While
    }
}

@Internal
object InsecureModifications {
    context(RLT.Node)
    fun <N : AST.Leaf> N.withRLT() = apply { this._rlt = this@Node }

    context (RLT.Node)
    fun <N : AST.NodeBase> N.withRLT() = apply { this._rlt = this@Node }
}