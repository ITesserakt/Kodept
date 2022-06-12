import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*
import java.math.BigInteger
import kotlin.io.path.Path

fun main() {
    System.setProperty("org.slf4j.simpleLogger.defaultLogLevel", "trace")

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
            ForeignFunctionResolver,
            OperatorDesugaring
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
            .then { parse(true) }
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
    ForeignFunctionResolver.exportFunction("kotlin.math.minus", BigInteger::minus)
    ForeignFunctionResolver.exportFunction<BigInteger, BigInteger, Boolean>("kotlin.math.eq") { a, b -> a == b }
    ForeignFunctionResolver.exportFunction("kotlin.math.times", BigInteger::times)

    result.programOutput.value().toEither().fold({
        with(code) { it.map { pr.processReport(it) } }
    }, { emptyList() }).joinToString("\n").let(::println)
}