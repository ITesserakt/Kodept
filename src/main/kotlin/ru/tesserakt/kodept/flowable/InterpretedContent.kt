package ru.tesserakt.kodept.flowable

import arrow.core.*
import arrow.eval.Eval
import io.github.oshai.kotlinlogging.KotlinLogging
import ru.tesserakt.kodept.FileInterpreter
import ru.tesserakt.kodept.InterpretationError.*
import ru.tesserakt.kodept.core.FileRelative
import ru.tesserakt.kodept.core.mapWithFilename
import ru.tesserakt.kodept.error.Report

private val logger = KotlinLogging.logger("[Compiler]")

class InterpretedContent(
    data: Flowable.Data.ErroneousAST,
    input: List<String>,
    private val computeLazy: Boolean = true,
) :
    Flowable<InterpretedContent.Data> {
    data class Data(override val programOutput: Eval<IorNel<Report, Pair<Any?, Int>>>) : Flowable.Data.Program

    private val run = data.ast.mapWithFilename { parsed ->
        logger.info { "Running ${this.name}..." }

        kotlin.runCatching {
            parsed.map { FileInterpreter().run(it.root, input) }
        }.fold({ it }, { Report(this, null, Report.Severity.ERROR, RuntimeException(it)).nel().leftIor() })
    }

    private fun <T> IorNel<T, *>.anyInLeft(predicate: (T) -> Boolean) =
        fold({ it.any(predicate) }, { false }, { _, _ -> false })

    private val mainAnalyze = run.mapWithFilename { res ->
        res.flatMap({ a, b -> a + b }) {
            when {
                it.mainFound && it.output == 0 -> it.rightIor()
                it.mainFound -> Ior.Both(
                    Report(this, null, Report.Severity.WARNING, WrongExitCode(it.output)).nel(), it
                )

                else -> Report(this, null, Report.Severity.ERROR, NoMainFunction).nel().leftIor()
            }
        }
    }

    private fun <T> ctor(): (() -> IorNel<Report, T>) -> Eval<IorNel<Report, T>> =
        if (computeLazy)
            { it -> Eval.later { it() } }
        else { it -> Eval.now(it()) }

    private val traverseForMain = {
        mainAnalyze.reduce { (acc, accFile), (next, nextFile) ->
            if (acc.anyInLeft { it.message == NoMainFunction }) FileRelative(
                acc.flatMap({ a, b -> a + b.filter { it.message == NoMainFunction } }) { next },
                nextFile
            ) else if (next.anyInLeft { it.message == NoMainFunction }) {
                FileRelative(
                    acc.flatMap({ a, b -> a + b }) { next },
                    nextFile
                )
            } else if (acc.anyInLeft { it.message is MultipleMain }) FileRelative(acc.mapLeft { nel ->
                nel.map {
                    if (it.message is MultipleMain) {
                        val msg = it.message as MultipleMain
                        Report(null, null, Report.Severity.ERROR, MultipleMain(msg.files + nextFile))
                    } else it
                }
            }, nextFile) else {
                FileRelative(acc.flatMap({ a, b -> a + b }) {
                    Report(null, null, Report.Severity.ERROR, MultipleMain(listOf(accFile, nextFile))).nel()
                        .leftIor()
                }, nextFile)
            }
        }.value
    }

    override val result: Data = Data(ctor<Pair<Any?, Int>>().invoke(traverseForMain andThen {
        it.map { state -> state.result to state.output }
    }))
}
