# start

'fun', IDENTIFIER, [parameterList](#parameterlist), [TypeGrammar.optional](typeGrammar.md#optional), '
{', [ [BlockLevelGrammar](blockLevel.md), (';' | '\n') ], { [BlockLevelGrammar](blockLevel.md), (';' | '\n') }, '}'

# parameterList

'(', [ [typed](#typed), ',' ], { [typed](#typed), ',' }, ')'

# typed

IDENTIFIER, [TypeGrammar.strict](typeGrammar.md#strict)