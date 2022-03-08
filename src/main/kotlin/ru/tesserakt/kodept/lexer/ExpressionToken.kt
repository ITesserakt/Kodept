package ru.tesserakt.kodept.lexer

import com.github.h0tk3y.betterParse.lexer.Token
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import com.github.h0tk3y.betterParse.lexer.literalToken
import com.github.h0tk3y.betterParse.lexer.regexToken

enum class ExpressionToken(val token: Token) {
    // Keywords
    FUN(literalToken("fun")),
    VAL(literalToken("val")),
    VAR(literalToken("var")),
    TRAIT(literalToken("trait")),
    STRUCT(literalToken("struct")),

    // CLASS(literalToken("class")), // stack only
    ENUM(literalToken("enum")),
    MODULE(literalToken("module")),
    EXTENSION(literalToken("extension")),

    // Symbols
    COMMA(literalToken(",")),
    SEMICOLON(literalToken(";")),
    LCURVE_BRACKET(literalToken("{")),
    RCURVE_BRACKET(literalToken("}")),
    LBRACKET(literalToken("[")),
    RBRACKET(literalToken("]")),
    LPAREN(literalToken("(")),
    RPAREN(literalToken(")")),
    TYPE_GAP(literalToken("_")),
    COLON(literalToken(":")),
    DOT(literalToken(".")),

    // Identifiers
    IDENTIFIER(regexToken("""_?[a-z]\w*""")),
    TYPE(regexToken("""_?[A-Z]\w*""")),

    // Literals
    DECIMAL(regexToken("""[-+]?([1-9][\d_]*\d|\d)""")),
    BINARY(regexToken("""0[bB](1[01_]*[01]|[01])""")),
    OCTAL(regexToken("""0[oO]([1-7][0-7_]*[0-7]|[0-7])""")),
    HEX(regexToken("""0[xX]([1-9A-F][\dA-F_]*[\dA-F]|[\dA-F])""")),
    CHAR(regexToken("""'[^']'""")),
    STRING(regexToken(""""(?:\\\\"|[^"])*"""")),
    FLOATING(regexToken("""[-+]?((\d+(\.\d*)?)|\.\d+)([eE][+-]?\d+)?""")),

    // Ignore
    WHITESPACE(regexToken("""\s+""", ignore = true)),
    NEWLINE(regexToken("[\r\n]+", ignore = true)),
    COMMENT(regexToken("//.*$", ignore = true)),
    MULTILINE_COMMENT(regexToken(Regex("/\\*.*\\*/", RegexOption.DOT_MATCHES_ALL), ignore = true)),

    // Operators
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
    EQUALS(literalToken("=")),
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
    NOT(literalToken("!")),

    SPACESHIP(literalToken("<=>")),
    LESS_EQUALS(literalToken("<=")),
    GREATER_EQUALS(literalToken(">=")),
    LESS(literalToken("<")),
    GREATER(literalToken(">"))
    ;

    init {
        token.name = this.name
    }
}


fun TokenMatch.noneMatched() = type == com.github.h0tk3y.betterParse.lexer.noneMatched