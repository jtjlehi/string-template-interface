# Language Specification

## TOKENS

OPEN        ::= `%{`
CLOSE       ::= `}`
ESCAPED_PER ::= `%%`
TEXT_STR    ::= (<ESCAPED_PER> | .)*
COLON       ::= `:`
FN_ARROW    ::= `->`
ASSIGN_OP   ::= `=`
OPEN_CURLY  ::= `{`
CLOSE_CURLY ::= `}`
COMMA       ::= `,`
DOT         ::= `.`
OPEN_PAREN  ::= `(`
CLOSE_PAREN ::= `)`
NEWLINE
WHITE_SPACE

## EBNF

BODY            ::= <FUNCTION>
                |   <TEXT>
TEXT            ::= <TEXT_STR> (<INSERT> | <TEXT_STR>)*
INSERT          ::= <OPEN> <VALUE> <CLOSE>
FUNCTION        ::= <OPEN_CURLY> (<DECL> <COMMA>)* <DECL>? <CLOSE_CURLY> <FN_ARROW> <NEWLINE>? <BODY>
                |   <DECL> <BODY>
DECL            ::= <VAR>
                |   <VAR> <ASSIGN_OP> <FN_VALUE>
FN_VALUE        ::= <VAR>
                |   <FUNCTION>
VAR             ::= <lower_char> (<char> | <number> | `_` | `-`)*
                |   `_` (<char> | <number> | `_` | `_`)+
                |   <ESCAPE_VAR>
VALUE           ::= <VAR>
                |   <ACCESS>
                |   <FN_CALL>
                |   <OPEN_PAREN> <VALUE> <CLOSE_PAREN>
ACCESS          ::= <VAR> <DOT> <ACCESS>
FN_CALL         ::= <VALUE> <VALUE>

