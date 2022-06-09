import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.ProgramState
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*
import java.time.Instant
import java.util.concurrent.ConcurrentHashMap
import kotlin.concurrent.thread

val megabytes = ConcurrentHashMap<Instant, Double>()

fun memoryStatThread() = thread(isDaemon = true) {
    val runtime = Runtime.getRuntime()
    val mb = 1024 * 1024.0

    while (true) {
        megabytes += Instant.now() to (runtime.totalMemory() - runtime.freeMemory()) / mb
        Thread.sleep(2)
    }
}

fun main() {
    memoryStatThread()
    val context = CompilationContext {
        loader = FileLoader()
        transformers = setOf(TypeSimplifier, InitializationTransformer, DereferenceTransformer, VariableScope)
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

    result.ast.forEach { it ->
        it.value.fold(
            { it.map { with(code) { pr.processReport(it) + "\n" } }.asSequence() },
            {
                val state = ProgramState(emptyList(), 0, emptyMap(), null)
                sequenceOf(
                    "Last expression in main: ${state.result}",
                    "Program exited with exit code: ${state.output}"
                )
            },
            { it, _ -> it.map { with(code) { pr.processReport(it) } }.asSequence() }
        ).joinToString("\n").let(::println)
    }

    println(
        "Maximum memory consumed: ${megabytes.maxBy { it.value }.value}\nAverage was: ${
            megabytes.map { it.value }.average()
        }"
    )
}