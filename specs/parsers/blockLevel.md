# start

[FunctionGrammar](functionGrammar.md#start) | [initialization](#initialization) | [varDecl](#vardecl)
| [assignment](#assignment) | [OperatorGrammar](operatorGrammar.md#start)

# varDecl

('VAL' | 'VAR'), [TermGrammar](termGrammar.md#variablereference), [TypeGrammar](typeGrammar.md#optional)

# initialization

[varDecl](#vardecl), '=', ([bracedDecl](#braceddecl) | [OperatorGrammar](operatorGrammar.md#start))

# assignment

[TermGrammar](termGrammar.md#start), ('+=' | '-=' | '*=' | '/=' | '%=' | '\**=' | '||=' | '&&=' | '|=' | '&=' | '^=' | '
='), ([bracedDecl](#braceddecl) | [OperatorGrammar](operatorGrammar.md#start))

# bracedDecl

'{', [start](#start), (';' | '\n'), { [start](#start), (';' | '\n') }, '}'