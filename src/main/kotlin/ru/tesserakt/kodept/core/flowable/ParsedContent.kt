package ru.tesserakt.kodept.core.flowable

import arrow.core.Eval
import arrow.core.IorNel
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report

context (CompilationContext)
class ParsedContent(flowable: Flowable.Data.ErroneousRawTree) : Flowable<ParsedContent.Data> {
    data class Data(
        override val forest: Eval<Map<Filename, ParseResult>>,
        override val ast: Sequence<FileRelative<IorNel<Report, AST>>>,
    ) : Flowable.Data.ErroneousAST, Flowable.Data.Forest

    private val trees = flowable.rlt.mapWithFilename { ior ->
        ior.map { AST(it.root.convert(), this) }
    }
    private val forest = Eval.later {
        trees.associate { it.filename to it.value }
    }

    override val result = Data(forest, trees)
}