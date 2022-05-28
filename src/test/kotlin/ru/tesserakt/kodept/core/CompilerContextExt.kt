package ru.tesserakt.kodept.core

import io.kotest.assertions.arrow.core.shouldBeLeft
import io.kotest.assertions.arrow.core.shouldBeRight
import ru.tesserakt.kodept.core.flowable.Flowable

context (CompilationContext.Scope)
fun Flowable.Data.ErroneousAST.shouldBeValid() = ast.mapWithFilename {
    it.toEither().shouldBeRight()
}.toList()

fun Flowable.Data.ErroneousAST.shouldBeInvalid() = ast.mapWithFilename {
    it.toEither().shouldBeLeft()
}.toList()