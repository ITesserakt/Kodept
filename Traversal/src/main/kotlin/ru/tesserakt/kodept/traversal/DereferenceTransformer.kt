package ru.tesserakt.kodept.traversal

import arrow.core.*
import arrow.core.continuations.eagerEffect
import arrow.core.continuations.ensureNotNull
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.walkDownTop
import ru.tesserakt.kodept.error.CompilerCrash
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import kotlin.reflect.KClass

private sealed interface FlowControl
private sealed interface FlowError : FlowControl
private object NotFound : FlowError
private data class Multiple(val list: List<AST.Named>) : FlowError
private object RecurseUp : FlowControl

object DereferenceTransformer : Transformer<AST.Reference>() {
    override val type: KClass<AST.Reference> = AST.Reference::class

    init {
        dependsOn(VariableScope)
    }

    private fun isInDereference(node: AST.Reference) = node.walkDownTop(::identity).any {
        if (it is AST.ObjectLevel) return false
        it is AST.Dereference
    }

    private fun <T : AST.Named> List<T>.onlyUnique(onEmpty: () -> FlowControl) = when (size) {
        0 -> onEmpty().left()
        1 -> this[0].right()
        else -> Multiple(this).left()
    }

    private fun AST.Reference.handleBlock(block: AST.ExpressionList) = block.expressions
        .filterIsInstance<AST.Referable>()
        .filter { it.name == this@handleBlock.name }
        .onlyUnique { RecurseUp }

    private fun AST.Reference.handleFunction(fn: AST.FunctionDecl): Either<FlowControl, AST.Referable> =
        if (fn.name == this.name)
            fn.right()
        else fn.params.filter { it.name == this.name }.onlyUnique { RecurseUp }

    private fun AST.Reference.handleVariable(v: AST.InitializedVar) =
        if (v.reference.name == this.name) v.right() else RecurseUp.left()

    private fun AST.Reference.handleModule(module: AST.ModuleDecl) = module.rest
        .filterIsInstance<AST.Referable>()
        .filter { it.name == this.name }
        .onlyUnique { NotFound }
        .mapLeft { it as FlowError }

    private fun AST.Reference.handleStruct(struct: AST.StructDecl) = struct.children()
        .filterIsInstance<AST.Referable>()
        .filter { it.name == this.name }
        .onlyUnique { RecurseUp }

    private fun AST.Reference.handle(node: AST.Node) = when (node) {
        is AST.ExpressionList -> handleBlock(node)
        is AST.FunctionDecl -> handleFunction(node)
        is AST.ForeignFunctionDecl -> handleForeignFunction(node)
        is AST.AbstractFunctionDecl -> handleAbstractFunction(node)
        is AST.InitializedVar -> handleVariable(node)
        is AST.ModuleDecl -> handleModule(node)
        is AST.StructDecl -> handleStruct(node)
        else -> RecurseUp.left()
    }

    private fun AST.Reference.handleAbstractFunction(node: AST.AbstractFunctionDecl): Either<FlowControl, AST.Referable> =
        if (node.name == this.name)
            node.right()
        else node.params.filter { it.name == this.name }.onlyUnique { RecurseUp }

    private fun AST.Reference.handleForeignFunction(node: AST.ForeignFunctionDecl): Either<FlowControl, AST.Referable> =
        if (node.name == this.name)
            node.right()
        else node.params.filter { it.name == this.name }.onlyUnique { RecurseUp }

    private fun AST.Reference.handleOrRecurseUp(node: AST.Node): Either<FlowError, AST.Referable> =
        handle(node).handleErrorWith { control ->
            when (control) {
                is Multiple -> control.left()
                NotFound -> NotFound.left()
                RecurseUp -> node.parent.rightIfNotNull { NotFound }.flatMap { handleOrRecurseUp(it) }
            }
        }

    context (Filename) private fun <T> Either<FlowError, T>.mapError(node: AST.Reference) = mapLeft { control ->
        when (control) {
            NotFound -> UnrecoverableError(
                node.rlt.position.nel(),
                Report.Severity.ERROR,
                SemanticError.UndeclaredUsage(node.name)
            )

            is Multiple -> UnrecoverableError(
                node.rlt.position.nel() + control.list.map { it.rlt.position },
                Report.Severity.ERROR,
                SemanticError.UndefinedUsage(node.name)
            )
        }
    }

    private fun AST.Type.findTypeIn(scope: AST.Node) = when (scope) {
        is AST.ModuleDecl -> scope.rest.filter { it.name == this.name }.onlyUnique { NotFound }
        is AST.EnumDecl -> scope.enumEntries.filter { it.name == this.name }.onlyUnique { NotFound }
        is AST.StructDecl -> scope.rest.filter { it.name == this.name }.onlyUnique { NotFound }
        else -> NotFound.left()
    }.mapLeft { it as FlowError }

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
    context(ReportCollector, Filename) override fun transform(node: AST.Reference) =
        eagerEffect<UnrecoverableError, AST.Node> {
            // FIXME think about dereferences
            if (isInDereference(node)) shift<Unit>(
                UnrecoverableError(
                    node.rlt.position.nel(),
                    Report.Severity.CRASH,
                    CompilerCrash("Dot access operator is unsupported")
                )
            )
            val parent = ensureNotNull(node.parent) {
                UnrecoverableError(
                    node.rlt.position.nel(),
                    Report.Severity.CRASH,
                    CompilerCrash("Invalid AST generated. Every reference should has parent")
                )
            }

            val referral = when (val context = node.resolutionContext) {
                null -> node.handleOrRecurseUp(parent).mapError(node).bind()
                else -> {
                    val start = if (context.fromRoot)
                        node.walkDownTop(::identity).filterIsInstance<AST.ModuleDecl>().first()
                    else node

                    val startMapped: Either<FlowError, AST.Named> = start.right().leftWiden()
                    val resolved = context.chain.fold(startMapped) { acc, type ->
                        acc.flatMap { type.findTypeIn(it) }
                    }.mapError(node).bind()

                    node.handle(resolved)
                        .mapLeft { if (it is RecurseUp) NotFound else it as FlowError }.mapError(node).bind()
                }
            }
            AST.ResolvedReference(node, referral)
        }
}