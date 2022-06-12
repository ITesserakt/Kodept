@file:Suppress("unused", "DuplicatedCode") @file:OptIn(Internal::class)

package ru.tesserakt.kodept.core

import arrow.core.NonEmptyList
import arrow.core.prependTo
import ru.tesserakt.kodept.core.Tree.SearchMode
import java.math.BigDecimal
import java.math.BigInteger
import kotlin.reflect.KClass
import kotlin.reflect.KMutableProperty0

data class AST(val root: Node, val filepath: Filepath) {
    init {
        walkThrough { node -> node.children().forEach { it.parent = node } }.forEach { _ -> }
    }

    fun <T> walkThrough(mode: SearchMode = SearchMode.LevelOrder, f: (Node) -> T) = root.walkTopDown(mode, f)

    fun flatten(mode: SearchMode = SearchMode.LevelOrder) = root.gatherChildren(mode)

    sealed interface Node : Tree<Node> {
        override var parent: Node?
        val rlt: RLT.Node

        override fun equals(other: Any?): Boolean
        override fun hashCode(): Int
        override fun toString(): String
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
    sealed interface TypeReferable : Named
    sealed interface TypeLike : Node
    sealed interface ParameterLike : Named {
        val type: TypeLike?
    }

    sealed class NodeBase : Node {
        final override var parent: Node? = null
        internal lateinit var _rlt: RLT.Node

        override val rlt: RLT.Node get() = _rlt

        @Internal
        abstract fun <A : Node?> replaceChild(old: A, new: A): Boolean

        protected inline fun <reified T : Node> MutableList<T>.replace(old: Node?, new: Node?) =
            old is T && remove(old) && new is T && add(new)

        protected inline fun <reified T : Node?> KMutableProperty0<T>.replace(old: Node?, new: Node?) =
            new is T && get() == old && true.apply { set(new) }
    }

    sealed class Leaf : Node {
        final override var parent: Node? = null
        final override fun children() = emptyList<Node>()
        internal lateinit var _rlt: RLT.Node

        override val rlt: RLT.Node get() = _rlt
    }

    data class Stub(private val prototype: Node) : Leaf(), Literal, Statement {
        override val rlt: RLT.Node get() = prototype.rlt
    }

    class Parameter(override val name: String, type: TypeLike) : NodeBase(), Referable, ParameterLike {
        override var type = type
            private set

        override fun children() = listOf(type)

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::type.replace(old, new)

        override fun toString(): String {
            return "Parameter(name='$name', type=$type)"
        }

        fun copy(name: String = this.name, type: TypeLike = this.type) = Parameter(name, type).also { it._rlt = _rlt }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is Parameter) return false

            if (name != other.name) return false
            if (type != other.type) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + type.hashCode()
            return result
        }

    }

    class InferredParameter(override val name: String, type: TypeLike?) : NodeBase(), Referable, ParameterLike {
        override var type = type
            private set

        override fun children() = listOfNotNull(type)

        fun copy(name: String = this.name, type: TypeLike? = this.type) = InferredParameter(name, type)

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::type.replace(old, new)

        override fun toString(): String {
            return "InferredParameter(name='$name', type=$type)"
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is InferredParameter) return false

            if (name != other.name) return false
            if (type != other.type) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + (type?.hashCode() ?: 0)
            return result
        }

    }

    data class FileDecl(private val _modules: MutableList<ModuleDecl>) : NodeBase() {
        val modules get() = NonEmptyList.fromListUnsafe(_modules)

        override fun children() = modules

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _modules.replace(old, new)

        constructor(modules: NonEmptyList<ModuleDecl>) : this(modules.toMutableList())

    }

    data class ModuleDecl(override val name: String, val global: Boolean, private val _rest: MutableList<TopLevel>) :
        NodeBase(), Named {
        val rest get() = _rest.toList()

        override fun children() = rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _rest.replace(old, new)

        constructor(name: String, global: Boolean, rest: Iterable<TopLevel>) : this(name, global, rest.toMutableList())

    }

    data class StructDecl(
        override val name: String,
        private val _alloc: MutableList<Parameter>,
        private val _rest: MutableList<StructLevel>,
    ) : NodeBase(), TopLevel, TypeReferable {
        val alloc get() = _alloc.toList()
        val rest get() = _rest.toList()

        override fun children() = alloc + rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            _alloc.replace(old, new) || _rest.replace(old, new)

        constructor(name: String, alloc: Iterable<Parameter>, rest: Iterable<StructLevel>) : this(
            name, alloc.toMutableList(), rest.toMutableList()
        )

    }

    data class ForeignStructDecl(override val name: String, val relatedWith: String) : Leaf(), TopLevel, TypeReferable

    data class EnumDecl(
        override val name: String,
        val stackBased: Boolean,
        private val _enumEntries: MutableList<EnumLevel>,
    ) : NodeBase(), TopLevel, TypeReferable {
        val enumEntries get() = _enumEntries.toList()
        override fun children() = enumEntries

        data class Entry(override val name: String) : Leaf(), EnumLevel, TypeReferable

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _enumEntries.replace(old, new)

        constructor(name: String, stackBased: Boolean, enumEntries: Iterable<EnumLevel>) : this(
            name, stackBased, enumEntries.toMutableList()
        )

    }

    data class TraitDecl(override val name: String, private val _rest: MutableList<TraitLevel>) : NodeBase(), TopLevel,
        TypeReferable {
        val rest get() = _rest.toList()
        override fun children() = rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _rest.replace(old, new)

        constructor(name: String, rest: Iterable<TraitLevel>) : this(name, rest.toMutableList())

    }

    class AbstractFunctionDecl(
        override val name: String, private val _params: MutableList<Parameter>, returns: TypeLike?,
    ) : NodeBase(), TraitLevel, Referable {
        var returns = returns
            private set
        val params get() = _params.toList()
        override fun children() = params + listOfNotNull(returns)

        override fun <A : Node?> replaceChild(old: A, new: A) = ::returns.replace(old, new) || _params.replace(old, new)

        fun copy(
            name: String = this.name,
            params: Iterable<Parameter> = this._params,
            returns: TypeLike? = this.returns,
        ) = AbstractFunctionDecl(name, params, returns).also { it._rlt = _rlt }

        override fun toString(): String {
            return "AbstractFunctionDecl(name='$name', params=$_params, returns=$returns)"
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is AbstractFunctionDecl) return false

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

        constructor(name: String, params: Iterable<Parameter>, returns: TypeLike?) : this(
            name, params.toMutableList(), returns
        )

    }

    class ForeignFunctionDecl(
        override val name: String,
        private val _params: MutableList<Parameter>,
        returns: TypeReference?,
        val descriptor: String,
        val action: ExportedFunction = ExportedFunction({ null }, emptyList(), Nothing::class),
    ) : NodeBase(), TopLevel, Referable {
        data class ExportedFunction(
            val action: (List<Any?>) -> Any?,
            val params: List<KClass<*>>,
            val returns: KClass<*>,
        )

        var returns = returns
            private set
        val params get() = _params.toList()

        override fun children() = params + listOfNotNull(returns)

        override fun <A : Node?> replaceChild(old: A, new: A) = ::returns.replace(old, new) || _params.replace(old, new)

        fun copy(
            name: String = this.name,
            params: Iterable<Parameter> = this._params,
            returns: TypeReference? = this.returns,
            descriptor: String = this.descriptor,
            action: ExportedFunction = this.action,
        ) = ForeignFunctionDecl(name, params, returns, descriptor, action).also { it._rlt = _rlt }

        val hasAction get() = action.returns != Nothing::class

        override fun toString(): String {
            val actionDescription =
                if (hasAction) ", action={in: ${action.params.map { it.simpleName }}, out: ${action.returns.simpleName}}"
                else ", action=<no action>"
            return "ForeignFunctionDecl(name='$name', params=$_params, returns=$returns)$actionDescription"
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is ForeignFunctionDecl) return false

            if (name != other.name) return false
            if (_params != other._params) return false
            if (descriptor != other.descriptor) return false
            if (action != other.action) return false
            if (returns != other.returns) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + _params.hashCode()
            result = 31 * result + descriptor.hashCode()
            result = 31 * result + action.hashCode()
            result = 31 * result + (returns?.hashCode() ?: 0)
            return result
        }

        constructor(
            name: String,
            params: Iterable<Parameter>,
            returns: TypeReference?,
            descriptor: String,
            action: ExportedFunction = ExportedFunction({ null }, emptyList(), Nothing::class),
        ) : this(
            name, params.toMutableList(), returns, descriptor, action
        )

    }

    class FunctionDecl(
        override val name: String, private val _params: MutableList<InferredParameter>,
        returns: TypeLike?, rest: Expression,
    ) : NodeBase(), TopLevel, StructLevel, TraitLevel, Referable {
        var returns = returns
            private set
        var rest = rest
            private set
        val params get() = _params.toList()
        override fun children() = params + listOfNotNull(returns) + listOf(rest)

        override fun <A : Node?> replaceChild(old: A, new: A) =
            ::rest.replace(old, new) || ::returns.replace(old, new) || _params.replace(old, new)

        fun copy(
            name: String = this.name,
            params: Iterable<InferredParameter> = this._params,
            returns: TypeLike? = this.returns,
            rest: Expression = this.rest,
        ) = FunctionDecl(name, params, returns, rest).also { it._rlt = _rlt }

        override fun toString(): String {
            return "FunctionDecl(name='$name', params=$_params, returns=$returns, rest=$rest)"
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is FunctionDecl) return false

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

        constructor(
            name: String,
            params: Iterable<InferredParameter>,
            returns: TypeLike?,
            rest: Expression,
        ) : this(name, params.toMutableList(), returns, rest)

    }

    class InitializedVar(reference: Reference, val mutable: Boolean, type: TypeLike?, expr: Expression) : NodeBase(),
        Referable {
        var reference: Reference = reference
            private set
        var type = type
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
            type: TypeLike? = this.type,
            expr: Expression = this.expr,
        ) = InitializedVar(reference, mutable, type, expr).also { it._rlt = _rlt }

        override fun toString(): String {
            return "InitializedVar(reference=${reference.name}, mutable=$mutable, type=$type expr=$expr)"
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is InitializedVar) return false

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

    }

    data class DecimalLiteral(val value: BigInteger) : Leaf(), Literal

    data class BinaryLiteral(val value: BigInteger) : Leaf(), Literal

    data class OctalLiteral(val value: BigInteger) : Leaf(), Literal

    data class HexLiteral(val value: BigInteger) : Leaf(), Literal

    data class CharLiteral(val value: Char) : Leaf(), Literal

    data class StringLiteral(val value: String) : Leaf(), Literal

    data class FloatingLiteral(val value: BigDecimal) : Leaf(), Literal

    data class TupleLiteral(private val _items: MutableList<Expression>) : NodeBase(), Literal {
        val items get() = _items.toList()

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _items.replace(old, new)

        override fun children() = items

        val arity = items.size

        companion object {
            val unit = TupleLiteral(mutableListOf())
        }

        constructor(items: Iterable<Expression>) : this(items.toMutableList())

    }

    sealed class BinaryOperator : NodeBase(), Expression {
        abstract var left: Expression
            protected set
        abstract var right: Expression
            protected set

        override fun children(): List<Node> = listOf(left, right)

        override fun <A : Node?> replaceChild(old: A, new: A) =
            new is Expression && (left == old && true.apply { left = new } || right == old && true.apply {
                right = new
            })

        abstract val kind: OperatorKind

        sealed interface OperatorKind
    }

    data class Mathematical(override var left: Expression, override var right: Expression, override val kind: Kind) :
        BinaryOperator() {
        enum class Kind : OperatorKind { Add, Sub, Mul, Div, Mod, Pow }
    }

    data class Logical(override var left: Expression, override var right: Expression, override val kind: Kind) :
        BinaryOperator() {
        enum class Kind : OperatorKind { Conjunction, Disjunction }
    }

    data class Comparison(override var left: Expression, override var right: Expression, override val kind: Kind) :
        BinaryOperator() {
        enum class Kind : OperatorKind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }
    }

    data class Binary(override var left: Expression, override var right: Expression, override val kind: Kind) :
        BinaryOperator() {
        enum class Kind : OperatorKind { And, Or, Xor }
    }

    sealed class UnaryOperator : NodeBase(), Expression {
        abstract var expr: Expression
            protected set

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            new is Expression && expr == old && true.apply { expr = new }

        override fun children(): List<Node> = listOf(expr)

    }

    data class Negation(override var expr: Expression) : UnaryOperator()
    data class Inversion(override var expr: Expression) : UnaryOperator()
    data class BitInversion(override var expr: Expression) : UnaryOperator()
    data class Absolution(override var expr: Expression) : UnaryOperator()
    data class Elvis(override var left: Expression, override var right: Expression) : BinaryOperator() {
        override val kind: OperatorKind = Kind

        private object Kind : OperatorKind
    }

    class Assignment(left: Lvalue, right: Expression) : NodeBase(), Statement {
        var left = left
            private set
        var right = right
            private set

        override fun children() = listOf(left, right)

        override fun <A : Node?> replaceChild(old: A, new: A) = ::left.replace(old, new) || ::right.replace(old, new)

        override fun toString(): String {
            return "Assignment(left=$left, right=$right)"
        }


        fun copy(left: Lvalue = this.left, right: Expression = this.right) =
            Assignment(left, right).also { it._rlt = _rlt }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is Assignment) return false

            if (left != other.left) return false
            if (right != other.right) return false

            return true
        }

        override fun hashCode(): Int {
            var result = left.hashCode()
            result = 31 * result + right.hashCode()
            return result
        }
    }

    data class ResolutionContext(val fromRoot: Boolean, val chain: List<Type>)

    open class Reference(override val name: String, val resolutionContext: ResolutionContext? = null) : Leaf(), Lvalue,
        Named {
        override fun toString(): String {
            return "Reference(name='$name', resolutionContext=$resolutionContext)"
        }

        fun copy(name: String = this.name, resolutionContext: ResolutionContext? = this.resolutionContext) =
            Reference(name, resolutionContext).also { it._rlt = _rlt }

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

    }

    class ResolvedReference(name: String, val referral: Referable, resolutionContext: ResolutionContext? = null) :
        Reference(name, resolutionContext) {
        override fun toString(): String {
            return "ResolvedReference(name='$name', referral=${referral.name}, resolutionContext=$resolutionContext)"
        }

        fun copy(
            name: String = this.name,
            referral: Referable = this.referral,
            resolutionContext: ResolutionContext? = this.resolutionContext,
        ) = ResolvedReference(name, referral, resolutionContext).also { it._rlt = _rlt }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is ResolvedReference) return false
            if (!super.equals(other)) return false

            if (referral.name != other.referral.name) return false

            return true
        }

        override fun hashCode(): Int {
            var result = super.hashCode()
            result = 31 * result + referral.name.hashCode()
            return result
        }
    }

    open class TypeReference(type: Type, val resolutionContext: ResolutionContext? = null) : NodeBase(), Lvalue,
        TypeLike {
        var type = type
            private set

        override fun children(): List<Node> = listOf(type)

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::type.replace(old, new)

        fun copy(type: Type = this.type, resolutionContext: ResolutionContext? = this.resolutionContext) =
            TypeReference(type, resolutionContext).also { it._rlt = _rlt }

        override fun toString(): String {
            return "TypeReference(resolutionContext=$resolutionContext, type=$type)"
        }

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

    }

    class ResolvedTypeReference(type: Type, val referral: TypeReferable, context: ResolutionContext? = null) :
        TypeReference(type, context) {
        override fun toString(): String {
            return "ResolvedTypeReference(type=$type, context=$resolutionContext, referral=${referral.name})"
        }

        fun copy(
            type: Type = this.type,
            referral: TypeReferable = this.referral,
            context: ResolutionContext? = this.resolutionContext,
        ) = ResolvedTypeReference(type, referral, context).also { it._rlt = _rlt }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is ResolvedTypeReference) return false
            if (!super.equals(other)) return false

            if (referral.name != other.referral.name) return false

            return true
        }

        override fun hashCode(): Int {
            var result = super.hashCode()
            result = 31 * result + referral.name.hashCode()
            return result
        }
    }

    class FunctionCall(
        reference: Reference,
        private val _params: MutableList<Expression>,
        val resolutionContext: ResolutionContext? = null,
    ) : NodeBase(), Lvalue {
        private var _reference = reference
        val reference get() = _reference
        val params get() = _params.toList()
        override fun children() = reference.prependTo(params)

        override fun <A : Node?> replaceChild(old: A, new: A) =
            ::_reference.replace(old, new) || _params.replace(old, new)

        override fun toString(): String {
            return "FunctionCall(params=$_params, resolutionContext=$resolutionContext, reference=$_reference)"
        }

        fun copy(
            reference: Reference = this._reference,
            params: Iterable<Expression> = this._params,
            resolutionContext: ResolutionContext? = this.resolutionContext,
        ) = FunctionCall(reference, params, resolutionContext).also { it._rlt = _rlt }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is FunctionCall) return false

            if (_params != other._params) return false
            if (resolutionContext != other.resolutionContext) return false
            if (_reference != other._reference) return false

            return true
        }

        override fun hashCode(): Int {
            var result = _params.hashCode()
            result = 31 * result + (resolutionContext?.hashCode() ?: 0)
            result = 31 * result + _reference.hashCode()
            return result
        }

        constructor(
            reference: Reference,
            params: Iterable<Expression>,
            resolutionContext: ResolutionContext? = null,
        ) : this(
            reference, params.toMutableList(), resolutionContext
        )

    }

    data class Dereference(override var left: Expression, override var right: Expression) : BinaryOperator(), Lvalue {
        override val kind: OperatorKind = Kind

        object Kind : OperatorKind
    }

    data class ExpressionList(private val _expressions: MutableList<BlockLevel>) : NodeBase(), Expression {
        val expressions get() = _expressions.toList()
        override fun children() = expressions

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _expressions.replace(old, new)

        constructor(expressions: Iterable<BlockLevel>) : this(expressions.toMutableList())

        fun copy(expressions: Iterable<BlockLevel> = this._expressions) =
            ExpressionList(expressions).also { it._rlt = _rlt }

    }

    sealed class TypeExpression : NodeBase()

    data class Type(override val name: String) : TypeExpression(), Named {
        override fun <A : Node?> replaceChild(old: A, new: A) = false

        override fun children() = emptyList<Node>()

        override fun toString() = name

    }

    data class TupleType(private val _items: MutableList<TypeLike>) : TypeExpression(), TypeLike {
        val items get() = _items.toList()

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _items.replace(old, new)

        override fun children() = items

        override fun toString() = items.joinToString(prefix = "(", postfix = ")")

        companion object {
            val unit = TupleType(mutableListOf())
        }

        constructor(items: Iterable<TypeLike>) : this(items.toMutableList())

        override fun equals(other: Any?): Boolean =
            other is TupleType && _items.containsAll(other._items) && other._items.containsAll(_items)

        override fun hashCode() = _items.hashCode()

    }

    data class UnionType(private val _items: MutableList<TypeLike>) : TypeExpression(), TypeLike {
        val items get() = NonEmptyList.fromListUnsafe(_items)

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _items.replace(old, new)

        override fun children() = items

        override fun toString() = items.joinToString(" | ", "(", ")")

        constructor(items: NonEmptyList<TypeLike>) : this(items.toMutableList())

        override fun equals(other: Any?): Boolean =
            other is UnionType && _items.containsAll(other._items) && other._items.containsAll(_items)

        override fun hashCode(): Int = _items.hashCode()


        fun copy(_items: NonEmptyList<TypeLike> = this.items) = UnionType(_items).also { it._rlt = rlt }
    }

    class IfExpr(
        condition: Expression,
        body: Expression,
        private val _elifs: MutableList<ElifExpr>,
        el: ElseExpr?,
    ) : NodeBase(), Expression {
        var condition = condition
            private set
        var body = body
            private set
        var el = el
            private set

        constructor(condition: Expression, body: Expression, elifs: Iterable<ElifExpr>, el: ElseExpr?) : this(
            condition, body, elifs.toMutableList(), el
        )

        fun copy(
            condition: Expression = this.condition,
            body: Expression = this.body,
            elifs: Iterable<ElifExpr> = this.elifs,
            el: ElseExpr? = this.el,
        ) = IfExpr(condition, body, elifs, el).also { it._rlt = _rlt }

        val elifs get() = _elifs.toList()

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            ::condition.replace(old, new) || ::body.replace(old, new) || _elifs.replace(old, new) || ::el.replace(
                old, new
            )

        override fun children() = listOf(condition, body) + elifs + listOfNotNull(el)

        override fun toString(): String {
            return "IfExpr(condition=$condition, body=$body, elifs=$_elifs, el=$el)"
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is IfExpr) return false

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


        class ElifExpr(condition: Expression, body: Expression) : NodeBase() {
            var condition = condition
                private set
            var body = body
                private set

            override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
                ::condition.replace(old, new) || ::body.replace(old, new)

            override fun children() = listOf(condition, body)

            fun copy(condition: Expression = this.condition, body: Expression = this.body) =
                ElifExpr(condition, body).also { it._rlt = _rlt }

            override fun toString(): String {
                return "ElifExpr(condition=$condition, body=$body)"
            }

            override fun equals(other: Any?): Boolean {
                if (this === other) return true
                if (other !is ElifExpr) return false

                if (condition != other.condition) return false
                if (body != other.body) return false

                return true
            }

            override fun hashCode(): Int {
                var result = condition.hashCode()
                result = 31 * result + body.hashCode()
                return result
            }

        }

        class ElseExpr(body: Expression) : NodeBase() {
            var body = body
                private set

            override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::body.replace(old, new)

            override fun children() = listOf(body)

            fun copy(body: Expression = this.body) = ElseExpr(body).also { it._rlt = _rlt }

            override fun toString(): String {
                return "ElseExpr(body=$body)"
            }

            override fun equals(other: Any?): Boolean {
                if (this === other) return true
                if (other !is ElseExpr) return false

                if (body != other.body) return false

                return true
            }

            override fun hashCode(): Int {
                return body.hashCode()
            }

        }
    }

    class WhileExpr(condition: Expression, body: Expression) : NodeBase(), Expression {
        var condition = condition
            private set
        var body = body
            private set

        fun copy(condition: Expression = this.condition, body: Expression = this.body) =
            WhileExpr(condition, body).also { it._rlt = _rlt }

        override fun <A : Node?> replaceChild(old: A, new: A) =
            ::condition.replace(old, new) || ::body.replace(old, new)

        override fun children() = listOf(condition, body)

        override fun toString(): String {
            return "WhileExpr(condition=$condition, body=$body)"
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other !is WhileExpr) return false

            if (condition != other.condition) return false
            if (body != other.body) return false

            return true
        }

        override fun hashCode(): Int {
            var result = condition.hashCode()
            result = 31 * result + body.hashCode()
            return result
        }

    }
}

object InsecureModifications {
    context(RLT.Node)
    fun <N : AST.Leaf> N.withRLT() = apply { this._rlt = this@Node }

    context (RLT.Node)
    fun <N : AST.NodeBase> N.withRLT() = apply { this._rlt = this@Node }
}

inline fun <reified T : RLT.Node> AST.Node.accessRLT() = rlt as? T