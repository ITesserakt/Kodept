import arrow.core.left
import arrow.core.right
import mu.KotlinLogging
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.core.asString
import ru.tesserakt.kodept.error.CompilerCrash
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportProcessor
import ru.tesserakt.kodept.traversal.*
import ru.tesserakt.kodept.traversal.inference.Function2LambdaTransformer
import ru.tesserakt.kodept.traversal.inference.TypeInferenceAnalyzer
import java.math.BigInteger
import kotlin.io.path.Path

fun main() {
    System.setProperty("org.slf4j.simpleLogger.defaultLogLevel", "trace")

    val logger = KotlinLogging.logger("[Compiler]")

    val context = CompilationContext {
        loader = FileLoader {
            path = Path("/home/tesserakt/IdeaProjects/Kodept/src/")
        }
        transformers = setOf(
            TypeSimplifier,
            InitializationTransformer,
            ReferenceResolver,
            VariableScope,
            TypeReferenceResolver,
            ForeignFunctionResolver,
            BinaryOperatorDesugaring,
            UnaryOperatorDesugaring,
            DereferenceEliminator,
            Function2LambdaTransformer
        )
        analyzers = setOf(
            moduleNameAnalyzer,
            moduleUniquenessAnalyzer,
            emptyBlockAnalyzer,
            variableUniqueness,
            objectUniqueness,
            TypeInferenceAnalyzer,
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

    ForeignFunctionResolver.exportFunction(
        { println(it[0]) },
        "kotlin.io.println",
        listOf(BigInteger::class),
        Unit::class
    )
    ForeignFunctionResolver.exportFunction<Unit>("kotlin.io.println") { println() }
    ForeignFunctionResolver.exportFunction<BigInteger>("kotlin.io.readInt") { readln().toBigInteger() }
    ForeignFunctionResolver.exportFunction("kotlin.math.plus", BigInteger::plus)
    ForeignFunctionResolver.exportFunction<BigInteger, BigInteger, Boolean>("kotlin.math.eq") { a, b -> a == b }
    ForeignFunctionResolver.exportFunction("kotlin.math.times", BigInteger::times)
    ForeignFunctionResolver.exportFunction("kotlin.math.minus", BigInteger::minus)
    ForeignFunctionResolver.exportFunction<BigInteger, BigInteger, Boolean>("kotlin.math.less") { a, b -> a < b }

    val pr = ReportProcessor {
        surrounding = 0
    }

    result.ast.forEach { (res, _) ->
        res.fold({ it.left() }, { it.right() }, { l, _ -> l.left() }).fold({
            with(code) {
                it.forEach {
                    when (it.severity) {
                        Report.Severity.NOTE -> logger.info { pr.processReport(it) }
                        Report.Severity.WARNING -> logger.warn { pr.processReport(it) }
                        Report.Severity.ERROR -> logger.error { pr.processReport(it) }
                        Report.Severity.CRASH -> (it.message as? CompilerCrash)?.run {
                            logger.error(this) { pr.processReport(it) }
                        } ?: logger.error(pr.processReport(it))
                    }
                }
            }
        }, { logger.debug("Competed successfully: ${it.asString()}") })
    }

//    result.programOutput.value().let(::println)
}