package ru.tesserakt.kodept.inference

import arrow.core.left
import arrow.core.nonEmptyListOf
import arrow.core.right
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.inference.ScopedDeclarations.Kind
import ru.tesserakt.kodept.inference.TypedRLTExpression.*

class Infer {
    private val scopes = mutableListOf<ScopedDeclarations<*>>()

    private inner class ScopeFactory<K : Kind>(val kind: K, val root: RLT.Scoping) {
        private val list = mutableListOf<RLT.Bind>()
        private val buildHooks = mutableListOf<() -> Unit>()
        private var parent: ScopeFactory<*>? = null

        fun <D : RLT.Bind> D.bindToScope() {
            list += this
        }

        fun <D : RLT.Bind> Iterable<D>.bindAll() {
            list += this
        }

        fun withParent(node: ScopeFactory<*>) = apply {
            parent = node
            node.buildHooks += {
                scopes += build()
                runHooks()
            }
        }

        fun extract() = this

        fun build(): ScopedDeclarations<K> = ScopedDeclarations(root, list, parent?.build(), kind)

        fun runHooks() {
            buildHooks.forEach { it() }
        }
    }

    private inline fun <T : Any> RLT.Module.beginGlobalScope(f: ScopeFactory<Kind.Global>.(RLT.Module) -> T): T {
        val factory = ScopeFactory(Kind.Global, this)
        return factory.run { f(this@beginGlobalScope) }.also {
            factory.runHooks()
            scopes += factory.build()
        }
    }

    context (ScopeFactory<Kind.Global>)
            private fun <T : Any, S : RLT.Scoping> S.beginObjectScope(f: ScopeFactory<Kind.Object>.(S) -> T): T {
        bindToScope()
        val factory = ScopeFactory(Kind.Object, this).withParent(extract())
        return factory.run { f(this@beginObjectScope) }
    }

    context (ScopeFactory<K>)
            private fun <T : Any, S : RLT.Scoping, K : Kind.InsideBlock> S.beginLocalScope(f: ScopeFactory<Kind.Local>.(S) -> T): T {
        bindToScope()
        val factory = ScopeFactory(Kind.Local, this).withParent(extract())
        return factory.run { f(this@beginLocalScope) }
    }

    context (ScopeFactory<K>)
            private fun <T : Any, S : RLT.Scoping, K : Kind.NonProtected> S.protect(f: ScopeFactory<Kind.Protected>.(S) -> T): T {
        bindToScope()
        val factory = ScopeFactory(Kind.Protected, this).withParent(extract())
        return factory.run { f(this@protect) }
    }

    private fun RLT.File.groupToScopes() {
        for (module in moduleList) {
            module.beginGlobalScope {
                for (node in module.rest) {
                    when (node) {
                        is RLT.Function.Bodied -> node.groupFn()
                        is RLT.Enum -> node.beginObjectScope {
                            node.rest.bindAll()
                        }

                        is RLT.Struct -> node.beginObjectScope {
                            node.varsToAlloc.bindAll()
                            node.rest.forEach {
                                when (it) {
                                    is RLT.Function.Bodied -> it.groupFn()
                                }
                            }
                        }

                        is RLT.Trait -> node.beginObjectScope {
                            node.rest.forEach {
                                when (it) {
                                    is RLT.Function.Abstract -> it.bindToScope()
                                    is RLT.Function.Bodied -> it.groupFn()
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    context (ScopeFactory<K>)
            private fun <K : Kind.NonProtected> RLT.Function.Bodied.groupFn(): Unit = protect {
        params.flatMap { it.params }.bindAll()
        body.groupBody()
    }

    context (ScopeFactory<K>)
            private fun <K : Kind.InsideBlock> RLT.Body.groupBody(): Unit = beginLocalScope {
        when (this@groupBody) {
            is RLT.Body.Block -> block.forEach { it.groupBlockNode() }
            is RLT.Body.Expression -> expression.bindToScope()
        }
    }

    context (ScopeFactory<Kind.Local>)
            private fun RLT.BlockLevelNode.groupBlockNode(): Unit = when (this) {
        is RLT.Body -> groupBody()
        is RLT.Function.Bodied -> groupFn()
        else -> bindToScope()
    }

    private fun ScopedDeclarations<*>.generateUniqueType(): KnownType.T {
        fun Int.expandToString(alphabet: List<Char> = ('a'..'z').toList()): String {
            if (this == 0) return alphabet[0].toString()
            var current = this
            val sb = StringBuilder()
            while (current > 0) {
                sb.append(alphabet[current % alphabet.size])
                current /= alphabet.size
            }
            return sb.reverse().toString()
        }

        val stringRepr = typesInUse.expandToString()
        typesInUse++
        return KnownType.T("`$stringRepr")
    }

    fun RLT.assignScopes(): List<ScopedDeclarations<*>> {
        scopes.clear()
        root.groupToScopes()
        return scopes.toList()
    }

    private fun ScopedDeclarations<*>.getType(node: RLT.Named) = when (node) {
        is RLT.Enum -> KnownType.Union(node.rest.map { KnownType.T(it.text.value()) })
        is RLT.Function -> KnownType.Fn(node.params.flatMap { it.params }
            .map { it.type?.known() ?: generateUniqueType() }, node.returnType?.known() ?: generateUniqueType())

        is RLT.MaybeTypedParameter -> node.type?.known() ?: generateUniqueType()
        is RLT.Module -> throw IllegalStateException()
        is RLT.Struct -> KnownType.Tuple(node.varsToAlloc.map { it.type.known() })
        is RLT.Trait -> KnownType.T(node.id.text.value())
        is RLT.Variable -> node.type?.known() ?: generateUniqueType()
        is RLT.InitializedAssignment -> node.lvalue.type?.let { KnownType.T(it.text.value()) } ?: generateUniqueType()
    }

    context (ScopedDeclarations<Kind.Local>)
            private fun RLT.ExpressionNode.annotateInScope(): TypedRLTExpression = when (this) {
        is RLT.BinaryOperation -> BinaryOperation(
            left.annotateInScope(),
            right.annotateInScope(),
            op,
            generateUniqueType()
        )

        is RLT.Body.Block -> when (block.size) {
            0 -> Block(nonEmptyListOf(TupleLiteral.unit.right()), generateUniqueType())
            else -> when (val last = block.last()) {
                is RLT.ExpressionNode ->
                    Block(nonEmptyListOf(last.annotateInScope().right()), generateUniqueType())

                is RLT.StatementNode ->
                    Block(nonEmptyListOf(last.left(), TupleLiteral.unit.right()), generateUniqueType())
            }
        }

        is RLT.Body.Expression -> expression.annotateInScope()
        is RLT.If -> If(
            condition.annotateInScope(),
            body.annotateInScope(),
            elifs.map { If.Elif(it.condition.annotateInScope(), it.body.annotateInScope(), generateUniqueType()) },
            el?.body?.annotateInScope() ?: BottomTypeLiteral,
            generateUniqueType()
        )

        is RLT.Literal.Floating -> FloatingLiteral(this, KnownType.Floating)
        is RLT.Literal.Number -> NumberLiteral(this, KnownType.Number)
        is RLT.Literal.Text -> when {
            isChar() -> TextLiteral(this, KnownType.Char)
            isString() -> TextLiteral(this, KnownType.String)
            else -> TextLiteral(this, KnownType.BottomType)
        }

        is RLT.Parameter -> id.annotateInScope()
        is RLT.Application -> Application(
            expr.annotateInScope(),
            params.map { tuple -> TupleLiteral(tuple.params.map { it.annotateInScope() }, generateUniqueType()) },
            generateUniqueType()
        )

        is RLT.Reference -> Reference(ref, getType(findByReference(this)!!))
        is RLT.ContextualReference -> TODO()
        is RLT.UnaryOperation -> UnaryOperation(expression.annotateInScope(), op, generateUniqueType())
        is RLT.While -> While(condition.annotateInScope(), body.annotateInScope(), generateUniqueType())
        is RLT.Literal.Tuple -> TupleLiteral(expressions.map { it.annotateInScope() }, generateUniqueType())
        is RLT.ParameterTuple -> TODO()
    }

    fun annotate() = scopes.flatMap {
        if (it.kind == Kind.Local) with(it as ScopedDeclarations<Kind.Local>) {
            it.declarations.filterIsInstance<RLT.ExpressionNode>().map { node -> node.annotateInScope() }
        } else {
            emptyList()
        }
    }
}