package ru.tesserakt.kodept.lexer

import com.github.h0tk3y.betterParse.lexer.Token
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import com.github.h0tk3y.betterParse.lexer.literalToken
import com.github.h0tk3y.betterParse.lexer.regexToken
import com.github.h0tk3y.betterParse.parser.Parser

enum class ExpressionToken(val token: Token) : Parser<TokenMatch> by token {
    // Keywords
    FUN(literalToken("fun")),
    VAL(literalToken("val")),
    VAR(literalToken("var")),
    TRAIT(literalToken("trait")),
    STRUCT(literalToken("struct")),
    IF(literalToken("if")),
    ELIF(literalToken("elif")),
    ELSE(literalToken("else")),
    MATCH(literalToken("match")),
    WHILE(literalToken("while")),
    CLASS(literalToken("class")), // stack only
    ENUM(literalToken("enum")),
    MODULE(literalToken("module")),
    EXTENSION(literalToken("extension")),

    // Symbols
    COMMA(literalToken(",")),
    SEMICOLON(literalToken(";")),
    LBRACE(literalToken("{")),
    RBRACE(literalToken("}")),
    LBRACKET(literalToken("[")),
    RBRACKET(literalToken("]")),
    LPAREN(literalToken("(")),
    RPAREN(literalToken(")")),
    TYPE_GAP(literalToken("_")),
    COLON(literalToken(":")),

    // Identifiers
    IDENTIFIER(regexToken("""_?[a-z]\w*""")),
    TYPE(regexToken("""_?[A-Z]\w*""")),

    // Literals
    BINARY(regexToken("""0[bB](1[01_]*[01]|[01])""")),
    OCTAL(regexToken("""0[oO]([1-7][0-7_]*[0-7]|[0-7])""")),
    HEX(regexToken("""0[xX]([1-9A-F][\dA-F_]*[\dA-F]|[\dA-F])""")),
    FLOATING(regexToken("""[-+]?((\d+(\.\d*)?)|\.\d+)([eE][+-]?\d+)?""")),

    //    DECIMAL(regexToken("""[-+]?([1-9][\d_]*\d|\d)""")),
    CHAR(regexToken("""'([^'\\]|\\'|\\\\)'""")),
    STRING(regexToken(""""(?:\\\\"|[^"])*"""")),

    // Ignore
    NEWLINE(regexToken("[\r\n]+", ignore = true)),
    WHITESPACE(regexToken("""\s+""", ignore = true)),
    COMMENT(regexToken("//.*$", ignore = true)),
    MULTILINE_COMMENT(regexToken(Regex("/\\*.*\\*/", RegexOption.DOT_MATCHES_ALL), ignore = true)),

    // Operators
    DOT(literalToken(".")),
    PLUS_EQUALS(literalToken("+=")),
    SUB_EQUALS(literalToken("-=")),
    TIMES_EQUALS(literalToken("*=")),
    DIV_EQUALS(literalToken("/=")),
    MOD_EQUALS(literalToken("%=")),
    POW_EQUALS(literalToken("**=")),
    PLUS(literalToken("+")),
    SUB(literalToken("-")),
    TIMES(literalToken("*")),
    DIV(literalToken("/")),
    MOD(literalToken("%")),
    POW(literalToken("**")),

    FLOW(literalToken("=>")),
    ELVIS(literalToken("?:")),

    OR_LOGIC_EQUALS(literalToken("||=")),
    AND_LOGIC_EQUALS(literalToken("&&=")),
    OR_LOGIC(literalToken("||")),
    AND_LOGIC(literalToken("&&")),
    OR_BIT_EQUALS(literalToken("|=")),
    AND_BIT_EQUALS(literalToken("&=")),
    XOR_BIT_EQUALS(literalToken("^=")),
    OR_BIT(literalToken("|")),
    AND_BIT(literalToken("&")),
    XOR_BIT(literalToken("^")),
    NOT_LOGIC(literalToken("!")),
    NOT_BIT(literalToken("~")),

    SPACESHIP(literalToken("<=>")),
    EQUIV(literalToken("==")),
    NOT_EQUIV(literalToken("!=")),
    LESS_EQUALS(literalToken("<=")),
    GREATER_EQUALS(literalToken(">=")),
    LESS(literalToken("<")),
    GREATER(literalToken(">")),

    EQUALS(literalToken("="));

    init {
        token.name = this.name
    }
}


fun TokenMatch.noneMatched() = type == com.github.h0tk3y.betterParse.lexer.noneMatched