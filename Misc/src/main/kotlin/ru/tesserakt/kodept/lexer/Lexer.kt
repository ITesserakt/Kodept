package ru.tesserakt.kodept.lexer

import com.github.h0tk3y.betterParse.lexer.DefaultTokenizer
import com.github.h0tk3y.betterParse.lexer.Tokenizer

private val default = DefaultTokenizer(ExpressionToken.values().map { it.token })

class Lexer : Tokenizer by default