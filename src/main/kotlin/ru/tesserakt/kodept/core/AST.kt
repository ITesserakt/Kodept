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

    sealed class Node : Tree<Node> {
        final override var parent: Node? = null
        val metadata: MetadataStore = emptyStore()

        abstract fun <A : Node?> replaceChild(old: A, new: A): Boolean

        protected inline fun <reified T : Node> MutableList<T>.replace(old: Node?, new: Node?) =
            old is T && remove(old) && new is T && add(new)

        protected inline fun <reified T : Node?> KMutableProperty0<T>.replace(old: Node?, new: Node?) =
            new is T && get() == old && true.apply { set(new) }
    }

    sealed class Leaf : Node() {
        final override fun children() = emptyList<Node>()

        final override fun <A : Node?> replaceChild(old: A, new: A): Boolean = false
    }

    class Parameter(val name: String, type: TypeExpression) : Node() {
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
    }

    class InferredParameter(val name: String, type: TypeExpression?) : Node() {
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
    }

    data class FileDecl(private val _modules: MutableList<ModuleDecl>) : Node() {
        val modules get() = NonEmptyList.fromListUnsafe(_modules)

        override fun children() = modules

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _modules.replace(old, new)

        constructor(modules: NonEmptyList<ModuleDecl>) : this(modules.toMutableList())
    }

    data class ModuleDecl(val name: String, val global: Boolean, private val _rest: MutableList<Node>) : Node() {
        val rest get() = _rest.toList()

        override fun children() = rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _rest.replace(old, new)

        constructor(name: String, global: Boolean, rest: Iterable<Node>) : this(name, global, rest.toMutableList())
    }

    data class StructDecl(val name: String, val _alloc: MutableList<Parameter>, private val _rest: MutableList<Node>) :
        Node() {
        val alloc get() = _alloc.toList()
        val rest get() = _rest.toList()

        override fun children() = alloc + rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            _alloc.replace(old, new) || _rest.replace(old, new)

        constructor(name: String, alloc: Iterable<Parameter>, rest: Iterable<Node>) : this(
            name, alloc.toMutableList(), rest.toMutableList()
        )
    }

    data class EnumDecl(val name: String, val stackBased: Boolean, private val _enumEntries: MutableList<Entry>) :
        Node() {
        val enumEntries get() = _enumEntries.toList()
        override fun children() = enumEntries

        data class Entry(val name: String) : Leaf()

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _enumEntries.replace(old, new)

        constructor(name: String, stackBased: Boolean, enumEntries: Iterable<Entry>) : this(
            name, stackBased, enumEntries.toMutableList()
        )
    }

    data class TraitDecl(val name: String, private val _rest: MutableList<Node>) : Node() {
        val rest get() = _rest.toList()
        override fun children() = rest

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _rest.replace(old, new)

        constructor(name: String, rest: Iterable<Node>) : this(name, rest.toMutableList())
    }

    class AbstractFunctionDecl(
        val name: String,
        private val _params: MutableList<Parameter>, returns: TypeExpression?,
    ) : Node() {
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
    }

    class FunctionDecl(
        val name: String,
        private val _params: MutableList<InferredParameter>, returns: TypeExpression?, rest: Node,
    ) : Node() {
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
            rest: Node = this.rest,
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
            rest: Node,
        ) : this(name, params.toMutableList(), returns, rest)
    }

    class VariableDecl(
        val name: String, val mutable: Boolean, type: TypeExpression?,
    ) : Node() {
        var type = type
            private set

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = ::type.replace(old, new)

        override fun children() = listOfNotNull(type)

        fun copy(name: String = this.name, mutable: Boolean = this.mutable, type: TypeExpression? = this.type) =
            VariableDecl(name, mutable, type)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as VariableDecl

            if (name != other.name) return false
            if (mutable != other.mutable) return false
            if (type != other.type) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + mutable.hashCode()
            result = 31 * result + (type?.hashCode() ?: 0)
            return result
        }

        override fun toString(): String {
            return "VariableDecl(name='$name', mutable=$mutable, type=$type)"
        }
    }

    class InitializedVar(decl: VariableDecl, expr: Node) : Node() {
        var decl = decl
            private set
        var expr = expr
            private set

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            ::decl.replace(old, new) || ::expr.replace(old, new)

        override fun children() = listOf(decl, expr)

        fun copy(decl: VariableDecl = this.decl, expr: Node = this.expr) = InitializedVar(decl, expr)

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as InitializedVar

            if (decl != other.decl) return false
            if (expr != other.expr) return false

            return true
        }

        override fun hashCode(): Int {
            var result = decl.hashCode()
            result = 31 * result + expr.hashCode()
            return result
        }

        override fun toString(): String {
            return "InitializedVar(decl=$decl, expr=$expr)"
        }
    }

    data class DecimalLiteral(val value: BigInteger) : Leaf()
    data class BinaryLiteral(val value: BigInteger) : Leaf()
    data class OctalLiteral(val value: BigInteger) : Leaf()
    data class HexLiteral(val value: BigInteger) : Leaf()
    data class CharLiteral(val value: Char) : Leaf()
    data class StringLiteral(val value: String) : Leaf()
    data class FloatingLiteral(val value: BigDecimal) : Leaf()

    data class TupleLiteral(private val _items: MutableList<Node>) : Node() {
        val items get() = _items.toList()

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _items.replace(old, new)

        override fun children() = items

        @Suppress("unused")
        val arity = items.size

        companion object {
            val unit = TupleLiteral(mutableListOf())
        }

        constructor(items: Iterable<Node>) : this(items.toMutableList())
    }

    sealed class BinaryOperator : Node() {
        abstract var left: Node
            protected set
        abstract var right: Node
            protected set

        override fun children(): List<Node> = listOf(left, right)

        override fun <A : Node?> replaceChild(old: A, new: A) =
            (::left as KMutableProperty0<Node>).replace(old, new) || (::right as KMutableProperty0<Node>).replace(
                old, new
            )
    }

    data class Mathematical(override var left: Node, override var right: Node, val kind: Kind) : BinaryOperator() {
        enum class Kind { Add, Sub, Mul, Div, Mod, Pow }
    }

    data class Logical(override var left: Node, override var right: Node, val kind: Kind) : BinaryOperator() {
        enum class Kind { Conjunction, Disjunction }
    }

    data class Comparison(override var left: Node, override var right: Node, val kind: Kind) : BinaryOperator() {
        enum class Kind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }
    }

    data class Binary(override var left: Node, override var right: Node, val kind: Kind) : BinaryOperator() {
        enum class Kind { And, Or, Xor }
    }

    sealed class UnaryOperator : Node() {
        abstract var expr: Node
            protected set

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean =
            (::expr as KMutableProperty0<Node>).replace(old, new)

        override fun children(): List<Node> = listOf(expr)
    }

    data class Negation(override var expr: Node) : UnaryOperator()
    data class Inversion(override var expr: Node) : UnaryOperator()
    data class BitInversion(override var expr: Node) : UnaryOperator()
    data class Absolution(override var expr: Node) : UnaryOperator()
    data class Elvis(override var left: Node, override var right: Node) : BinaryOperator()
    data class Assignment(override var left: Node, override var right: Node) : BinaryOperator()
    data class ResolutionContext(val fromRoot: Boolean, val chain: List<TypeReference>)
    data class Reference(val name: String, val resolutionContext: ResolutionContext? = null) : Leaf()
    class TypeReference(type: TypeExpression, val resolutionContext: ResolutionContext? = null) : Node() {
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
    }

    data class FunctionCall(
        val reference: Node,
        private val _params: MutableList<Node>,
        val resolutionContext: ResolutionContext? = null,
    ) : Node() {
        val params get() = _params.toList()
        override fun children() = reference.prependTo(params)
        override fun <A : Node?> replaceChild(old: A, new: A) = _params.replace(old, new)

        constructor(reference: Node, params: Iterable<Node>, resolutionContext: ResolutionContext? = null) : this(
            reference, params.toMutableList(), resolutionContext
        )
    }

    data class TermChain(private val _terms: MutableList<Node>) : Node() {
        val terms get() = _terms.toList()
        override fun children() = terms
        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _terms.replace(old, new)

        constructor(terms: Iterable<Node>) : this(terms.toMutableList())
    }

    data class ExpressionList(private val _expressions: MutableList<Node>) : Node() {
        val expressions get() = _expressions.toList()
        override fun children() = expressions
        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _expressions.replace(old, new)

        constructor(expressions: Iterable<Node>) : this(expressions.toMutableList())
    }

    sealed class TypeExpression : Node()

    data class Type(val name: String) : TypeExpression() {
        override fun <A : Node?> replaceChild(old: A, new: A) = false

        override fun children() = emptyList<Node>()

        override fun toString() = name
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
    }

    data class UnionType(private val _items: MutableList<TypeExpression>) : TypeExpression() {
        val items get() = NonEmptyList.fromListUnsafe(_items)

        override fun <A : Node?> replaceChild(old: A, new: A): Boolean = _items.replace(old, new)

        override fun children() = items

        override fun toString() = items.joinToString(" | ", "(", ")")

        constructor(items: NonEmptyList<TypeExpression>) : this(items.toMutableList())
    }

    class IfExpr(
        condition: Node,
        body: Node,
        private val _elifs: MutableList<ElifExpr>,
        el: ElseExpr?,
    ) : Node() {
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

        class ElifExpr(condition: Node, body: Node) : Node() {
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
        }

        class ElseExpr(body: Node) : Node() {
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
        }
    }

    class WhileExpr(condition: Node, body: Node) : Node() {
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
    }
}