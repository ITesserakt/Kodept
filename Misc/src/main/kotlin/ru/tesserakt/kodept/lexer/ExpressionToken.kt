package ru.tesserakt.kodept.lexer

import com.github.h0tk3y.betterParse.lexer.Token
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import com.github.h0tk3y.betterParse.lexer.literalToken
import com.github.h0tk3y.betterParse.lexer.regexToken
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.parser.softKeyword

enum class ExpressionToken(val token: Token) : Parser<TokenMatch> by token {
    // Ignore
    COMMENT(regexToken("""//( |.)*""", ignore = true)),
    MULTILINE_COMMENT(regexToken(Regex("/\\*.*\\*/", RegexOption.DOT_MATCHES_ALL), ignore = true)),
    NEWLINE(regexToken("[\r\n]+", ignore = true)),
    WHITESPACE(regexToken("""\s+""", ignore = true)),

    // Keywords
    FUN(literalToken("fun ")),
    VAL(literalToken("val ")),
    VAR(literalToken("var ")),
    IF(literalToken("if ")),
    ELIF(literalToken("elif ")),
    ELSE(literalToken("else ")),
    MATCH(literalToken("match ")),
    WHILE(literalToken("while ")),
    MODULE(literalToken("module ")),
    EXTENSION(literalToken("extension ")),

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
    DOUBLE_COLON(literalToken("::")),
    COLON(literalToken(":")),

    // Identifiers
    IDENTIFIER(regexToken("""_?[a-z][\w_]*""")),
    TYPE(regexToken("""_?[A-Z][\w_]*""")),

    // Literals
    BINARY(regexToken("""0[bB](1[01_]*[01]|[01])""")),
    OCTAL(regexToken("""0[oO]([1-7][0-7_]*[0-7]|[0-7])""")),
    HEX(regexToken("""0[xX]([1-9A-F][\dA-F_]*[\dA-F]|[\dA-F])""")),
    FLOATING(regexToken("""[-+]?((\d+(\.\d*)?)|\.\d+)([eE][+-]?\d+)?""")),

    //    DECIMAL(regexToken("""[-+]?([1-9][\d_]*\d|\d)""")),
    CHAR(regexToken("""'([^'\\]|\\'|\\\\)'""")),
    STRING(regexToken(""""(?:\\\\|\\"|[^"\n])*"""")),

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
    DIV(literalToken("/")),
    MOD(literalToken("%")),
    POW(literalToken("**")),
    TIMES(literalToken("*")),

    FLOW(literalToken("=>")),
    ELVIS(literalToken("?:")),
    NOT_EQUIV(literalToken("!=")),

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
    LESS_EQUALS(literalToken("<=")),
    GREATER_EQUALS(literalToken(">=")),
    LESS(literalToken("<")),
    GREATER(literalToken(">")),

    EQUALS(literalToken("="));

    init {
        token.name = this.name
    }

    companion object {
        // Soft keywords
        val ABSTRACT = softKeyword("abstract")
        val TRAIT = softKeyword("trait")
        val STRUCT = softKeyword("struct")
        val CLASS = softKeyword("class") // stack only
        val ENUM = softKeyword("enum")
        val FOREIGN = softKeyword("foreign")
        val TYPE_ALIAS = softKeyword("type")
    }
}