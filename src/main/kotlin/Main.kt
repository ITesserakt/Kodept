import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.FileInterpreter
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*

fun memoryStats() {
    val mb = 1024 * 1024
    // get Runtime instance
    val instance = Runtime.getRuntime()
    println("***** Heap utilization statistics [MB] *****\n")
    // available memory
    println("Total Memory: " + instance.totalMemory() / mb)
    // free memory
    println("Free Memory: " + instance.freeMemory() / mb)
    // used memory
    println("Used Memory: " + (instance.totalMemory() - instance.freeMemory()) / mb)
    // Maximum available memory
    println("Max Memory: " + instance.maxMemory() / mb)
}

fun main() {
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
                sequenceOf(FileInterpreter().run(it.root, emptyList()).output.toString())
            },
            { it, _ -> it.map { with(code) { pr.processReport(it) } }.asSequence() }
        ).joinToString("\n").let(::println)
    }
}