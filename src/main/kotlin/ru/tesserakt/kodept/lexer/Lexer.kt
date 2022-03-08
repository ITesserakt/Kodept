package ru.tesserakt.kodept.lexer

import com.github.h0tk3y.betterParse.lexer.DefaultTokenizer
import com.github.h0tk3y.betterParse.lexer.Tokenizer

class Lexer : Tokenizer by DefaultTokenizer(ExpressionToken.values().map { it.token })