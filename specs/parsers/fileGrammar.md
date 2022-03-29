# start

([moduleStatement](#modulestatement), { [moduleStatement](#modulestatement) })
| [globalModuleStatement](#globalmodulestatement)

# moduleStatement

'module', IDENTIFIER, '{', { [TopLevelGrammar](topLevelGrammar.md#start) }, '}'

# globalModuleStatement

'module', IDENTIFIER, '=>', { [TopLevelGrammar](topLevelGrammar.md#start) }