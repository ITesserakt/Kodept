import arrow.core.identity
import io.arrow.core.fst
import ru.tesserakt.kodept.analyzer.ModuleAnalyzer
import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.transformer.ASTScopeTagger
import kotlin.system.measureTimeMillis

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

    val processor = ReportProcessor()

    measureTimeMillis {
        result.ast.flatMap {
            it.value.fold(::identity, { emptyList() }, ::fst).map {
                with(code) { processor.processReport(it) }
            }
        }.forEach(::println)
    }.let(::println)

    List(1000) {
        measureTimeMillis {
            result.ast.flatMap {
                it.value.fold(::identity, { emptyList() }, ::fst).map {
                    with(code) { processor.processReport(it) }
                }
            }.count()
        }
    }.minOf { it }.let(::println)
}