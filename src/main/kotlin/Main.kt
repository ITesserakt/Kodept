import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*

fun main() {
    val context = CompilationContext {
        loader = FileLoader()
        transformers = setOf(TypeSimplifier, InitializationTransformer, DereferenceTransformer, VariableScope)
        analyzers = setOf(
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
            .then { parse(true) }
            .then { dropUnusedInfo() }
            .then { analyze() }
            .also { sources.bind().holder }
    }

    val pr = ReportProcessor {
        surrounding = 0
    }

    result.ast.take(1).forEach { it ->
        it.value.fold(
            { it.map { with(code) { pr.processReport(it) + "\n" } }.asSequence() },
            { "".asSequence() },
            { it, _ -> it.map { with(code) { pr.processReport(it) } }.asSequence() }
        ).joinToString("\n").let(::println)
    }
}