package ru.tesserakt.kodept.traversal

import arrow.core.*
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.ReportMessage
import ru.tesserakt.kodept.lexer.CodePoint
import kotlin.reflect.KClass
import kotlin.reflect.safeCast

sealed interface ControlSwitching
data class UnrecoverableError(val crashReport: Report) : ControlSwitching {
    companion object {
        context (Filename)
                operator fun invoke(
            position: NonEmptyList<CodePoint>,
            severity: Report.Severity,
            message: ReportMessage,
        ) =
            UnrecoverableError(Report(this@Filename, position, severity, message))
    }
}
object Skip : ControlSwitching

interface Transformer<A : AST.Node> {
    val type: KClass<A>

    context (ReportCollector, Filename)
    fun transform(node: A): EagerEffect<UnrecoverableError, out AST.Node>
}

context (ReportCollector, Filename)
fun <A : AST.Node> Transformer<A>.transformOrSkip(node: AST.Node) =
    type.safeCast(node)?.let { transform(it) } ?: eagerEffect { node }

abstract class Analyzer {
    protected open val walkMode = Tree.SearchMode.LevelOrder
    protected abstract fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit>

    fun ReportCollector.analyzeWithCaching(ast: AST): EagerEffect<UnrecoverableError, Unit> {
        treeCache.computeIfAbsent(ast) { it.flatten(walkMode).toList() }
        return analyze(ast)
    }

    protected fun AST.fastFlatten(mode: Tree.SearchMode = walkMode, filtering: (AST.Node) -> Boolean = { true }) =
        treeCache[this]?.filter(filtering)?.asSequence() ?: flatten(mode).filter(filtering)

    companion object {
        private var treeCache = hashMapOf<AST, List<AST.Node>>()
    }
}

fun <T> unwrap(f: ReportCollector.() -> EagerEffect<out UnrecoverableError, T>) = with(ReportCollector()) {
    f(this).fold({ (it.crashReport.nel() + collectedReports).leftIor() }, {
        if (!hasReports) it.rightIor()
        else if (hasErrors) definitelyCollected.leftIor()
        else Ior.Both(definitelyCollected, it)
    })
}