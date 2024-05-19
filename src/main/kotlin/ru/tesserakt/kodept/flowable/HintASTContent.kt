package ru.tesserakt.kodept.flowable

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.FileRelative
import ru.tesserakt.kodept.core.withFilename

class HintASTContent(a: Flowable.Data.Source) : Flowable<HintASTContent.Data> {
    data class Data(
        override val unprocessedAST: Sequence<FileRelative<AST?>>,
    ) : Flowable.Data.UnprocessedAST

    private val unprocessedAST = a.source.map { it.withFilename { hint } }

    override val result = Data(unprocessedAST)
}