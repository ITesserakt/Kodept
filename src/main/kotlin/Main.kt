import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.FileInterpreter
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*
import java.time.Instant
import java.util.concurrent.ConcurrentHashMap
import kotlin.concurrent.thread

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
        loader = FileLoader()
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

    val (result, code) = context flow {
        val sources = readSources()
        sources
            .then { tokenize() }
            .then { parse(true) }
            .then { dropUnusedInfo() }
            .then { analyze() }
            .also { sources.bind().holder }
    }

    val pr = ReportProcessor {
        surrounding = 0
    }

    ForeignFunctionResolver.exportFunction<Unit>("kotlin.io.println") { println() }
    ForeignFunctionResolver.exportFunction<String, Unit>("kotlin.io.println") { println(it) }

    result.ast.forEach { it ->
        println("Processing ${it.filename}...")

        it.value.fold(
            { it.map { with(code) { pr.processReport(it) + "\n" } }.asSequence() },
            {
                val state = FileInterpreter().run(it.root, emptyList())
                sequenceOf(
                    "Last expression in main: ${state.result}",
                    "Program exited with exit code: ${state.output}"
                )
            },
            { it, _ -> it.map { with(code) { pr.processReport(it) } }.asSequence() }
        ).joinToString("\n").let(::println)
    }

    alive = false
    println(
        """Maximum memory consumed: ${megabytes.values.maxOrNull()}
           |Average was: ${megabytes.map { it.value }.average()}
           |Total consumed: ${megabytes.map { it.value }.zipWithNext(Double::minus).sum()}
        """.trimMargin()
    )
}