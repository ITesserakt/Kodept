import ru.tesserakt.kodept.FileLoader
import ru.tesserakt.kodept.lexer.Lexer
import ru.tesserakt.kodept.lexer.noneMatched

fun main() {
    val lexer = Lexer()

    FileLoader {
        extension = "kt"
    }.loadSources()
        .map { it.bufferedReader().readText() }
        .flatMap { lexer.tokenize(it) }
        .filter { it.noneMatched() }
        .forEach(::println)
}