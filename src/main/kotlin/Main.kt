import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*
import java.time.Instant
import java.util.concurrent.ConcurrentHashMap
import kotlin.concurrent.thread
import kotlin.io.path.Path

val megabytes = ConcurrentHashMap<Instant, Double>()
var alive = true

fun memoryStatThread() = thread(isDaemon = true) {
    val runtime = Runtime.getRuntime()
    val mb = 1024 * 1024.0

    while (alive) {
        megabytes += Instant.now() to (runtime.totalMemory() - runtime.freeMemory()) / mb
        Thread.sleep(1)
    }
}

fun main() {
    memoryStatThread()
    val context = CompilationContext {
        loader = FileLoader {
            this.path = Path("/home/tesserakt/IdeaProjects/Kodept/src/test/")
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

    result.programOutput.value().toEither().fold({
        with(code) { it.map { pr.processReport(it) } }
    }, { emptyList() }).joinToString("\n").let(::println)
    alive = false

    println(
        """Maximum memory consumed: ${megabytes.values.maxOrNull()}
           |Average was: ${megabytes.map { it.value }.average()}
           |Total consumed: ${megabytes.map { it.value }.zipWithNext(Double::minus).sum()}
        """.trimMargin()
    )
}