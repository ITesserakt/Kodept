import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*
import ru.tesserakt.kodept.traversal.inference.TypeAssigner
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
            BinaryOperatorDesugaring,
            UnaryOperatorDesugaring
        )
        analyzers = setOf(
            moduleNameAnalyzer,
            moduleUniquenessAnalyzer,
            emptyBlockAnalyzer,
            variableUniqueness,
            objectUniqueness,
            TypeAssigner,
        )
    }

    val (result, code) = context workflow {
        val sources = readSources()
        sources
            .then { tokenize() }
            .then { parse(true) }
            .then { dropUnusedInfo() }
            .then { analyze() }
//            .then { interpret() }
            .also { sources.bind().holder }
    }

    ForeignFunctionResolver.exportFunction({ println(it[0]) }, "kotlin.io.println", listOf(String::class), Unit::class)
    ForeignFunctionResolver.exportFunction<Unit>("kotlin.io.println") { println() }
    ForeignFunctionResolver.exportFunction<BigInteger>("kotlin.io.readInt") { readln().toBigInteger() }
    ForeignFunctionResolver.exportFunction("kotlin.math.plus", BigInteger::plus)
    ForeignFunctionResolver.exportFunction<BigInteger, BigInteger, Boolean>("kotlin.math.eq") { a, b -> a == b }
    ForeignFunctionResolver.exportFunction("kotlin.math.times", BigInteger::times)
    ForeignFunctionResolver.exportFunction<BigInteger, BigInteger, Boolean>("kotlin.math.less") { a, b -> a < b }

    val pr = ReportProcessor {
        surrounding = 0
    }

    result.ast.map { (res, _) ->
        res.fold({ with(code) { it.map { pr.processReport(it) } } },
            { emptyList() },
            { r, _ -> with(code) { r.map { pr.processReport(it) } } })
    }.joinToString("\n").let(::println)

//    result.programOutput.value().let(::println)
}