import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.moduleNameAnalyzer
import ru.tesserakt.kodept.traversal.moduleUniquenessAnalyzer

fun main() {
    val context = CompilationContext {
        loader = FileLoader()
        analyzers = listOf(moduleNameAnalyzer, moduleUniquenessAnalyzer)
    }

    val (code, result) = context flow {
        val sources = readSources()
        sources.bind().holder to sources
            .then { tokenize() }
            .then { parse() }
            .then { applyTransformations() }
            .then { analyze() }
            .bind()
    }

    val pr = ReportProcessor {
        surrounding = 0
    }

    result.ast.forEach { it ->
        it.value.toEither().fold(
            { it.map { with(code) { pr.processReport(it) } }.asSequence() },
            { it.flatten(Tree.SearchMode.LevelOrder).map { it::class.simpleName } }
        ).joinToString("\n") { it.toString() }.let(::println)
    }
}