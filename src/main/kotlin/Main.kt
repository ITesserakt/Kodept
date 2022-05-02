import ru.tesserakt.kodept.analyzer.ModuleAnalyzer
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.transformer.ASTScopeTagger

operator fun String.times(n: Int) = repeat(n)

fun main() {
    val context = CompilationContext {
        loader = MemoryLoader.singleSnippet("module A { }")
        transformers = listOf(::ASTScopeTagger)
        analyzers = listOf(ModuleAnalyzer())
    }
    val result = context.flow {
        acquireContent()
            .tokenize()
            .parse()
            .transform()
            .analyze()
    }

    result.toList().let(::println)
}

//private fun prettyError(report: Report, compilationFlow: CompilationFlow): String {
//    val maxIndexLength = report.point?.maxOf { log10((it.line + 2).toFloat()).toInt() + 1 } ?: 0
//    val codeWindows = report.point.orEmpty().map { point ->
//        val fileStream = compilationFlow.acquireContents().first { it.name == report.file }.contents
//        val from = (point.line - 2).coerceAtLeast(0)
//
//        fileStream.reader().useLines { lines ->
//            lines.drop(from).take(2).withIndex().joinToString("\n") { (index, str) ->
//                val realIndex = index + 1 + from
//                val lineNumber = "    %${maxIndexLength}d | ".format(realIndex)
//                lineNumber + if (realIndex == point.line)
//                    "$str\n${" " * (point.position + lineNumber.length - 1)}^--- ${report.message.additionalMessage}"
//                else str
//            }
//        }
//    }
//
//    return "$report\n${codeWindows.joinToString("\n")}"
//}