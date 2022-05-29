import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*

fun main() {
    val context = CompilationContext {
        loader = FileLoader()
        transformers = listOf(TypeSimplifier, DereferenceTransformer)
        analyzers = listOf(
            moduleNameAnalyzer,
            moduleUniquenessAnalyzer,
            emptyBlockAnalyzer,
            variableUniqueness,
            objectUniqueness
        )
    }

    val (result, code) = context flow {
        val sources = readSources()
        sources
            .then { tokenize() }
            .then { parse() }
            .then { abstract() }
            .then { applyTransformations() }
            .then { analyze() }
            .also { sources.bind().holder }
    }

    val pr = ReportProcessor {
        surrounding = 0
    }

    result.ast.forEach { it ->
        it.value.fold(
            { it.map { with(code) { pr.processReport(it) + "\n" } }.asSequence() },
            { "".asSequence() },
            { it, _ -> it.map { with(code) { pr.processReport(it) } }.asSequence() }
        ).joinToString("\n").let(::println)
    }
}