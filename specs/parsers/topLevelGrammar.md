# start

[structStatement](#structstatement) | [traitStatement](#traitstatement) | [enumStatement](#enumStatement)
| [functionStatement](functionGrammar.md#start)

# structStatement

'struct', TYPE, '{', [ [ObjectLevelGrammar](objectLevelGrammar.md#start), (';' | '\n') ],
{ [ObjectLevelGrammar](objectLevelGrammar.md#start), (';' | '\n') }, '}'

# traitStatement

'trait', TYPE, '{', [ [ObjectLevelGrammar](objectLevelGrammar.md#start), (';' | '\n') ],
{ [ObjectLevelGrammar](objectLevelGrammar.md#start), (';' | '\n') }, '}'

# enumStatement

'enum', ('struct' | 'class'), TYPE, '{', TYPE, ',', { TYPE, ',' } '}'