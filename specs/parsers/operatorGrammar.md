# start

[elvis](#elvis)

# elvis

[logicExpr](#logicexpr), [ '?:', [elvis](#elvis) ]

# logicExpr

[bitExpr](#bitexpr), { ('&&' | '||'), [bitExpr](#bitexpr) }

# bitExpr

[cmpExpr](#cmpexpr), { ('&' | '|' | '^'), [cmpExpr](#cmpexpr) }

# cmpExpr

[compoundExpr](#compoundexpr), { ('<' | '>'), [compoundExpr](#compoundexpr) }

# compoundExpr

[complexExpr](#complexexpr), { ('<=' | '==' | '!=' | '>='), [complexExpr](#complexexpr) }

# complexExpr

[addExpr](#addexpr), { '<=>', [addExpr](#addexpr) }

# addExpr

[mulExpr](#mulexpr), { ('+' | '-'), [mulExpr](#mulexpr) }

# mulExpr

[powExpr](#powexpr), { ('*' | '/' | '%'), [powExpr](#powexpr) }

# powExpr

[topExpr](#topexpr), [ '**', [powExpr](#powexpr) ]

# topExpr

('-', [topExpr](#topexpr)) | ('!', [topExpr](#topexpr)) | ('~', [topExpr](#topexpr)) | ('+', [topExpr](#topexpr))
| [atom](#atom)

# atom

('(', [start](#start), ')') | [ExpressionGrammar](expression.md)