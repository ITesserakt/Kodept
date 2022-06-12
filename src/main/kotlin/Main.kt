import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*
import java.math.BigInteger
import kotlin.io.path.Path

fun main() {
    val context = CompilationContext {
        loader = FileLoader {
            path = Path("/home/tesserakt/IdeaProjects/Kodept/src/")
        }
        transformers = setOf(
            TypeSimplifier,
            InitializationTransformer,
            DereferenceTransformer,
            VariableScope,
            TypeDereferenceTransformer,
            ForeignFunctionResolver
        )
        analyzers = setOf(
            moduleNameAnalyzer,
            moduleUniquenessAnalyzer,
            emptyBlockAnalyzer,
            variableUniqueness,
            objectUniqueness
        )
    }

    val (result, code) = context workflow {
        val sources = readSources()
        sources
            .then { tokenize() }
            .then { parse(false) }
            .then { dropUnusedInfo() }
            .then { analyze() }
            .then { interpret() }
            .also { sources.bind().holder }
    }

    val pr = ReportProcessor {
        surrounding = 0
    }

    ForeignFunctionResolver.exportFunction({ println(it[0]) }, "kotlin.io.println", listOf(String::class), Unit::class)
    ForeignFunctionResolver.exportFunction<Unit>("kotlin.io.println") { println() }
    ForeignFunctionResolver.exportFunction<BigInteger>("kotlin.io.readInt") { readln().toBigInteger() }

    result.programOutput.value().toEither().fold({
        with(code) { it.map { pr.processReport(it) } }
    }, { emptyList() }).joinToString("\n").let(::println)
}