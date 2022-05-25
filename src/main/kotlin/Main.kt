import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.emptyBlockAnalyzer
import ru.tesserakt.kodept.traversal.moduleNameAnalyzer
import ru.tesserakt.kodept.traversal.moduleUniquenessAnalyzer
import ru.tesserakt.kodept.traversal.typeSimplifier

fun main() {
    val context = CompilationContext {
        loader = FileLoader()
        transformers = listOf(typeSimplifier)
        analyzers = listOf(moduleNameAnalyzer, moduleUniquenessAnalyzer, emptyBlockAnalyzer)
    }

    val (code, result) = context flow {
        val sources = readSources()
        sources.bind().holder to sources
            .then { tokenize() }
            .then { parse() }
            .then { abstract() }
            .then { applyTransformations() }
            .then { analyze() }
            .bind()
    }

    val pr = ReportProcessor {
        surrounding = 0
    }

    result.ast.forEach { it ->
        it.value.fold(
            { it.map { with(code) { pr.processReport(it) } }.asSequence() },
            { "Nothing special".split(" ").asSequence() },
            { it, _ -> it.map { with(code) { pr.processReport(it) } }.asSequence() }
        ).joinToString("\n").let(::println)
    }
}