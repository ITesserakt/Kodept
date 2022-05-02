import arrow.core.identity
import io.arrow.core.fst
import ru.tesserakt.kodept.analyzer.ModuleAnalyzer
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.transformer.ASTScopeTagger
import kotlin.system.measureNanoTime

fun main() {
    val context = CompilationContext {
        loader = FileLoader()
        transformers = listOf(::ASTScopeTagger)
        analyzers = listOf(ModuleAnalyzer())
    }
    val result = with(context) {
        val sources = acquireContent()

        sources + sources.tokenize()
            .parse()
            .transform()
            .analyze()
    }

    val processor = ReportProcessor()
    measureNanoTime {
        result.b.result.flatMap {
            it.value.fold(::identity, { emptyList() }, ::fst).map {
                with(result.a) {
                    processor.cacheText()
                    processor.processReport(it)
                }
            }
        }.toList().forEach(::println)
    }.let { println("Elapsed: ${it / 1000.0}us") }
}