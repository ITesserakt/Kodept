package ru.tesserakt.kodept.traversal

import arrow.core.*
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.traversal.TypeDereferenceTransformer.handle
import kotlin.reflect.KClass

sealed interface FlowControl
private sealed interface FlowError : FlowControl
private object NotFound : FlowError
private data class Multiple(val list: List<AST.Named>) : FlowError
private object RecurseUp : FlowControl

private interface Resolver<T : AST.Named, R : AST.Node> {
    fun T.handle(node: AST.Node): Either<FlowControl, R>

    fun T.handleOrRecurseUp(node: AST.Node): Either<FlowError, R> = handle(node).handleErrorWith { control ->
        when (control) {
            is Multiple -> control.left()
            NotFound -> NotFound.left()
            RecurseUp -> node.parent.rightIfNotNull { NotFound }.flatMap { handleOrRecurseUp(it) }
        }
    }

    context (Filepath)
    fun followContext(context: AST.ResolutionContext, node: T) =
        if (context.fromRoot && context.chain.isNotEmpty()) {
            node.walkDownTop(::identity).filterIsInstance<AST.FileDecl>()
                .first().modules.filter { it.name == context.chain.first().name }.onlyUnique { NotFound }
                .let { m ->
                    val module = m.handleError {
                        node.walkDownTop(::identity)
                            .filterIsInstance<AST.ModuleDecl>().first()
                    }.widen<_, AST.Named, _>()
                    context.chain.drop(if (m.isRight()) 1 else 0).fold(module) { acc, next ->
                        acc.flatMap { next.handle(it) }
                    }
                }
        } else if (context.fromRoot) {
            val m = node.walkDownTop(::identity).filterIsInstance<AST.ModuleDecl>().first()
            val module: Either<FlowControl, AST.Named> = m.right()
            context.chain.fold(module) { acc, next -> acc.flatMap { next.handle(it) } }
        } else {
            val firstNode: Either<FlowControl, AST.Named> = node.right()
            context.chain.fold(firstNode) { acc, next -> acc.flatMap { next.handle(it) } }
        }
}

private fun <T : AST.Node> isInDereference(node: T) = node.walkDownTop(::identity).any {
    if (it is AST.ObjectLevel) return false
    it is AST.Dereference
}

private fun <T : AST.Named> List<T>.onlyUnique(onEmpty: () -> FlowControl) = when (size) {
    0 -> onEmpty().left()
    1 -> this[0].right()
    else -> Multiple(this).left()
}

context (Filepath) private inline fun <T, N : AST.Node> Either<FlowError, T>.mapError(node: N, getName: (N) -> String) =
    mapLeft { control ->
        when (control) {
            NotFound -> UnrecoverableError(
                node.rlt.position.nel(), Report.Severity.ERROR, SemanticError.UndeclaredUsage(getName(node))
            )

            is Multiple -> UnrecoverableError(
                node.rlt.position.nel() + control.list.map { it.rlt.position },
                Report.Severity.ERROR,
                SemanticError.UndefinedUsage(getName(node))
            )
        }
    }

context (Filepath) private fun <T, N> Either<FlowError, T>.mapError(node: N)
        where N : AST.WithResolutionContext, N : AST.Named = mapError(node) {
    val ctx = node.context
    if (ctx != null)
        (if (ctx.fromRoot) "::" else "") + "${ctx.chain.joinToString("::", postfix = "::") { it.name }}${node.name}"
    else node.name
}

object DereferenceTransformer : Transformer<AST.Reference>(), Resolver<AST.Reference, AST.Referable> {
    override val type: KClass<AST.Reference> = AST.Reference::class

    init {
        dependsOn(VariableScope, BinaryOperatorDesugaring, ForeignFunctionResolver, UnaryOperatorDesugaring)
    }

    private fun AST.Reference.handleBlock(block: AST.ExpressionList) =
        block.expressions.filterIsInstance<AST.Referable>().filter { it.name == this@handleBlock.name }
            .onlyUnique { RecurseUp }

    private fun AST.Reference.handleFunction(fn: AST.FunctionDecl): Either<FlowControl, AST.Referable> =
        if (fn.name == this.name) fn.right()
        else fn.params.filter { it.name == this.name }.onlyUnique { RecurseUp }

    private fun AST.Reference.handleVariable(v: AST.InitializedVar) =
        if (v.reference.name == this.name) v.right() else RecurseUp.left()

    private fun AST.Reference.handleModule(module: AST.ModuleDecl) =
        module.rest.filterIsInstance<AST.Referable>().filter { it.name == this.name }.onlyUnique { NotFound }
            .mapLeft { it as FlowError }

    private fun AST.Reference.handleStruct(struct: AST.StructDecl) =
        struct.children().filterIsInstance<AST.Referable>().filter { it.name == this.name }.onlyUnique { RecurseUp }

    override fun AST.Reference.handle(node: AST.Node) = when (node) {
        is AST.ExpressionList -> handleBlock(node)
        is AST.FunctionDecl -> handleFunction(node)
        is AST.ForeignFunctionDecl -> handleForeignFunction(node)
        is AST.AbstractFunctionDecl -> handleAbstractFunction(node)
        is AST.InitializedVar -> handleVariable(node)
        is AST.ModuleDecl -> handleModule(node)
        is AST.StructDecl -> handleStruct(node)
        is AST.TraitDecl -> handleTrait(node)
        else -> RecurseUp.left()
    }

    private fun AST.Reference.handleTrait(node: AST.TraitDecl): Either<FlowControl, AST.Referable> =
        node.children().filterIsInstance<AST.Referable>().filter { it.name == this.name }.onlyUnique { RecurseUp }

    private fun AST.Reference.handleAbstractFunction(node: AST.AbstractFunctionDecl): Either<FlowControl, AST.Referable> =
        if (node.name == this.name) node.right()
        else node.params.filter { it.name == this.name }.onlyUnique { RecurseUp }

    private fun AST.Reference.handleForeignFunction(node: AST.ForeignFunctionDecl): Either<FlowControl, AST.Referable> =
        if (node.name == this.name) node.right()
        else node.params.filter { it.name == this.name }.onlyUnique { RecurseUp }

    /**
     * > This works except for [AST.Dereference]: we should know type of the left branch to find proper reference for the right branch
     *
     * Every reference is in block: [AST.ExpressionList] or [AST.FunctionDecl] or [AST.InitializedVar] or [AST.ModuleDecl]
     *
     * 1) reference without context:
     *     1) in expression list - `{ x }` - find initialized vars in this block or function with this name or recurse upper
     *
     *     2) in function - `fun x => x` - check if function name equals to ref or recurse upper
     *
     *     3) in variable - `val x` - always accepts
     *
     *     4) in module - `module A => fun x {}` - find function with name
     *
     *     5) in struct parameters - `struct X(x, y)` - find by name
     *
     * 2) if reference with context then:
     *
     *     1) just global context - `::x` - recurse upper to module and 1.4)
     *
     *     2) global context with continue - `::x::y::...::z` - recurse upper to module, 1.4) then 1.x) without recursion upper
     *
     *     3) context - `x::y::...::z` - 1.x) without recursion upper
     */
    context(ReportCollector, Filepath) override fun transform(node: AST.Reference): EagerEffect<UnrecoverableError, AST.Node> {
        // FIXME think about dereferences

        val referral = when (val context = node.context) {
            null -> node.handleOrRecurseUp(node.parent).mapError(node)
            else -> followContext(context, node)
                .flatMap { point -> node.handle(point) }
                .mapLeft { if (it !is FlowError) NotFound else it }
                .mapError(node)
        }
        return eagerEffect {
            AST.ResolvedReference(node.name, referral.bind(), node.context)
        }
    }

    override val traverseMode: Tree.SearchMode = Tree.SearchMode.Preorder

    val contract = Contract<AST.Reference, Depended> {
        require(this !is AST.ResolvedReference)
        "$this should not be in AST. Try adding $DereferenceTransformer as a dependency in $it"
    }
}

object TypeDereferenceTransformer : Transformer<AST.TypeReference>(), Resolver<AST.Type, AST.TypeReferable> {
    override val type: KClass<AST.TypeReference> = AST.TypeReference::class

    init {
        dependsOn(BinaryOperatorDesugaring, UnaryOperatorDesugaring)
    }

    context(ReportCollector, Filepath) override fun transform(node: AST.TypeReference): EagerEffect<UnrecoverableError, AST.Node> {
        val parent = node.parent

        val referral = when (val context = node.context) {
            null -> node.type.handleOrRecurseUp(parent).mapError(node)
            else -> followContext(context, node.type)
                .flatMap { point -> node.type.handle(point) }
                .mapLeft { if (it !is FlowError) NotFound else it }
                .mapError(node)
        }
        return eagerEffect {
            AST.ResolvedTypeReference(node.type, referral.bind(), node.context)
        }
    }

    override fun AST.Type.handle(node: AST.Node): Either<FlowControl, AST.TypeReferable> = when (node) {
        is AST.ModuleDecl -> node.rest.filterIsInstance<AST.TypeReferable>().filter { it.name == this.name }
            .onlyUnique { NotFound }

        is AST.EnumDecl -> node.enumEntries.mapNotNull { it as? AST.TypeReferable }.filter { it.name == this.name }
            .onlyUnique { RecurseUp }

        is AST.TraitDecl, is AST.StructDecl -> if ((node as AST.TypeReferable).name == this.name) node.right() else RecurseUp.left()

        else -> RecurseUp.left()
    }

    val contract = Contract<AST.TypeReference, Depended> {
        require(this !is AST.ResolvedTypeReference)
        "$this should not be in AST. Try adding $TypeDereferenceTransformer as a dependency in $it"
    }
}