package ru.tesserakt.kodept.core

import arrow.core.NonEmptyList
import kotlin.reflect.KMutableProperty0

@DslMarker
annotation class ASTBuilder

@ASTBuilder
interface BasicScope

interface Builder<S : BasicScope, N : AST.Node> {
    fun scope(): S

    fun S.assemble(): N
}

abstract class ScopeWithRest<B : Builder<*, N>, N : AST.Node> : BasicScope {
    val etc: MutableList<NotAssembled<B, *, N>> = mutableListOf()
}

data class NotAssembled<B : Builder<S, N>, S : BasicScope, N : AST.Node>(val builder: B, val scope: S) {
    operator fun invoke() = with(builder) {
        scope.assemble()
    }
}

inline fun <reified T : Builder<S, N>, S : BasicScope, N : AST.Node> node(block: S.() -> Unit) =
    with(T::class.objectInstance!!) {
        NotAssembled(this, scope().apply(block))
    }

inline fun <reified T : Builder<S, N>, S : BasicScope, N : AST.Node> Filepath.generateAST(block: S.() -> Unit) =
    AST(node<T, _, _>(block).invoke(), this)

object File : Builder<File.Scope, AST.FileDecl> {
    class Scope : ScopeWithRest<Module, AST.ModuleDecl>() {
        private val globalRefs = mutableListOf<KMutableProperty0<Boolean>>()

        fun module(name: String, block: Module.Scope.() -> Unit) = node<Module, _, _> {
            block()
            this.name = name
            global = this@Scope.etc.isEmpty()
            this@Scope.globalRefs += ::global
        }.also {
            if (etc.size > 1)
                globalRefs.forEach { prop -> prop.set(false) }
            etc += it
        }.let { it() }
    }

    override fun scope(): Scope = Scope()
    override fun Scope.assemble(): AST.FileDecl = AST.FileDecl(NonEmptyList.fromListUnsafe(etc).map { it() })
}

object Module : Builder<Module.Scope, AST.ModuleDecl> {
    class Scope : BasicScope {
        lateinit var name: String
        var global = false
        val etc =
            mutableListOf<NotAssembled<out Builder<out BasicScope, out AST.TopLevel>, out BasicScope, out AST.TopLevel>>()

        fun struct(name: String, block: Struct.Scope.() -> Unit) = node<Struct, _, _> {
            block()
            this.name = name
        }.also { etc += it }.let { it() }
    }

    override fun scope(): Scope = Scope()
    override fun Scope.assemble(): AST.ModuleDecl = AST.ModuleDecl(name, global, etc.map { it() })
}

object Struct : Builder<Struct.Scope, AST.StructDecl> {
    class Scope : ScopeWithRest<Builder<BasicScope, AST.StructLevel>, AST.StructLevel>() {
        lateinit var name: String
        val params: MutableList<AST.Parameter> = mutableListOf()

//        fun parameter(name: String, type: )
    }

    override fun scope(): Scope = Scope()

    override fun Scope.assemble(): AST.StructDecl = AST.StructDecl(name, params, etc.map { it() })
}

interface ContextualBuilder<S : BasicScope, N : AST.WithResolutionContext> : Builder<S, N> {
    object Root {
        operator fun invoke() = AST.ResolutionContext(true, emptyList())
    }

    operator fun Root.div(path: String) = AST.ResolutionContext(true, listOf(AST.Type(path)))
    operator fun String.div(path: String) = AST.ResolutionContext(false, listOf(this, path).map(AST::Type))
    operator fun String.invoke() = AST.ResolutionContext(false, listOf(AST.Type(this)))
    operator fun AST.ResolutionContext.div(path: String) = AST.ResolutionContext(fromRoot, chain + AST.Type(path))
}

object TypeReference : ContextualBuilder<TypeReference.Scope, AST.TypeReference> {
    class Scope : BasicScope {
        lateinit var type: String
        var context: AST.ResolutionContext? = null
    }

    override fun scope(): Scope = Scope()

    override fun Scope.assemble() = AST.TypeReference(AST.Type(type), context)
}

object Parameter

fun foo() {
    val root = Filepath("").generateAST<File, _, _> {
    }
}