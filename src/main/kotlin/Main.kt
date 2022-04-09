import ru.tesserakt.kodept.Compiler
import ru.tesserakt.kodept.FileCache
import ru.tesserakt.kodept.FileLoader
import java.io.File

fun main() {
    val compiler = Compiler(FileLoader())

    compiler.cache { FileCache(File(it.removeSuffix(".kd") + ".kdc")) }.count()
}