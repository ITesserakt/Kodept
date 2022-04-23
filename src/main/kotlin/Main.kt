import ru.tesserakt.kodept.analyzer.ModuleAnalyzer
import ru.tesserakt.kodept.core.Compiler
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.transformer.ASTScopeTagger
import kotlin.io.path.Path
import kotlin.math.log10

operator fun String.times(n: Int) = repeat(n)

fun main() {
    val compiler = Compiler(FileLoader {
        path = Path("/home/tesserakt/IdeaProjects/Kodept/src/test/resources")
    }) {
        transformers = listOf(::ASTScopeTagger)
    }

    val ast = compiler.transform()
    ModuleAnalyzer().analyze(ast).value().map { report ->
        val maxIndexLength = report.point.maxOf { log10((it.line + 2).toFloat()).toInt() + 1 }
        val codeWindows = report.point.map { point ->
            val fileStream = compiler.acquireContents().first { it.name == report.file }.contents
            val from = (point.line - 2).coerceAtLeast(0)

            fileStream.reader().useLines { lines ->
                lines.drop(from).take(2).withIndex().joinToString("\n") { (index, str) ->
                    val realIndex = index + 1 + from
                    val lineNumber = "    %${maxIndexLength}d | ".format(realIndex)
                    lineNumber + if (realIndex == point.line)
                        "$str\n${" " * (point.position + lineNumber.length - 1)}^--- ${report.message.additionalMessage}"
                    else str
                }
            }
        }

        "$report\n${codeWindows.joinToString("\n    ...\n")}"
    }.forEach(::println)
}