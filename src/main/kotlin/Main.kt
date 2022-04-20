import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Compiler
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.transformer.ASTScopeTagger
import ru.tesserakt.kodept.transformer.acceptTransform
import ru.tesserakt.kodept.visitor.DrawProcessor
import ru.tesserakt.kodept.visitor.accept

fun main() {
    val compiler = Compiler(FileLoader())

    val ast = compiler.parse().map { it.toParsedOrThrow().value }

    ast.map { AST(it.root.acceptTransform(ASTScopeTagger()), it.fileName) }
        .map { it.root.accept(DrawProcessor()) }
        .joinToString("\n")
        .let(::println)
}