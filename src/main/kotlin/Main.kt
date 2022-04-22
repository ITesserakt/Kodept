import ru.tesserakt.kodept.core.Compiler
import ru.tesserakt.kodept.core.FileCache
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.transformer.ASTScopeTagger
import java.io.File

fun main() {
    val compiler = Compiler(FileLoader()) {
        transformers = listOf(::ASTScopeTagger)
    }

    compiler.cache { FileCache(File(it.removeSuffix(".kd") + ".json")) }.toList()
}