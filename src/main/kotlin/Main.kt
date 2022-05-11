import arrow.core.nel
import ru.tesserakt.kodept.analyzer.ModuleAnalyzer
import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.transformer.ASTScopeTagger
import ru.tesserakt.kodept.visitor.DrawProcessor
import ru.tesserakt.kodept.visitor.accept

fun main() {
    val context = CompilationContext {
        loader = FileLoader()
        transformers = listOf(::ASTScopeTagger)
        analyzers = listOf(ModuleAnalyzer())
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
        println(it.value.toEither().fold({
            it.map {
                with(code) { pr.processReport(it) }
            }
        }) { it.root.accept(DrawProcessor()).nel() }.joinToString("\n"))
    }
}