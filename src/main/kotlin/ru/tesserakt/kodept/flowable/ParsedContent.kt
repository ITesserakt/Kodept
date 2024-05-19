package ru.tesserakt.kodept.flowable

import arrow.eval.Eval
import arrow.core.IorNel
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.convert
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report

context (CompilationContext)
class ParsedContent(flowable: Flowable.Data.ErroneousRawTree) : Flowable<ParsedContent.Data> {
    data class Data(
        override val forest: Eval<Map<Filepath, ParseResult>>,
        override val ast: Sequence<FileRelative<IorNel<Report, AST>>>,
    ) : Flowable.Data.ErroneousAST, Flowable.Data.Forest

    private val trees = flowable.rlt.mapWithFilename { ior ->
        ior.map { AST(it.root.convert(), this) }
    }
    private val forest = Eval.later {
        trees.associate { it.filepath to it.value }
    }

    override val result = Data(forest, trees)
}