# Typhon Language Grammar Specification (EBNF)

This document defines the formal grammar of the Typhon programming language using Extended Backus-Naur Form (EBNF). Typhon is a statically typed programming language based on Python 3, combining Python's elegant syntax with compile-time type checking.

## EBNF Notation Used in This Document

- `::=` defines a production rule (non-terminal symbol)
- `|` indicates alternatives (choice)
- `[]` indicates optional elements (0 or 1 occurrence)
- `{}` indicates repetition (0 or more occurrences)
- `()` is used for grouping
- Terminal symbols are enclosed in single quotes: `'+'`
- Non-terminal symbols are in lowercase with underscores: `expression_list`
- UPPERCASE symbols represent lexical tokens: `NEWLINE`, `INDENT`, `DEDENT`

## 1. Module Structure and Top-Level Elements

```ebnf
# A module consists of zero or more statements
module ::= {statement}

# A statement can be a simple statement, compound statement, or declaration
statement ::= simple_statement | compound_statement | declaration

# A simple statement is a single statement terminated by a newline or semicolon
simple_statement ::= (
    expression_statement |
    assignment_statement |
    augmented_assignment_statement |
    import_statement |
    from_import_statement |
    assert_statement |
    pass_statement |
    del_statement |
    global_statement |
    nonlocal_statement |
    return_statement |
    break_statement |
    continue_statement |
    raise_statement |
    yield_statement
) (NEWLINE | ';')

# A compound statement has a header and a suite
compound_statement ::= (
    if_statement |
    while_statement |
    for_statement |
    try_statement |
    with_statement |
    match_statement |
    async_for_statement |
    async_with_statement
)

# A declaration defines a new name
declaration ::= (
    function_declaration |
    class_declaration |
    type_declaration
)

# A block of indented statements
suite ::= NEWLINE INDENT {statement} DEDENT
```

## 2. Import Statements

```ebnf
# Simple import statement
import_statement ::= 'import' dotted_as_names

# Import with 'from' prefix
from_import_statement ::= 'from' (dotted_name | '.' {'.'}
                         'import' ('*' | '(' import_as_names ')' | import_as_names)

# One or more dotted names with optional aliases
dotted_as_names ::= dotted_as_name {',' dotted_as_name}

# A dotted name with an optional alias
dotted_as_name ::= dotted_name ['as' identifier]

# One or more import names with optional aliases
import_as_names ::= import_as_name {',' import_as_name} [',']

# An import name with an optional alias
import_as_name ::= identifier ['as' identifier]

# A name with dots (e.g., module.submodule)
dotted_name ::= identifier {'.' identifier}
```

## 3. Declarations

```ebnf
# Function declaration
function_declaration ::= ['async'] 'def' identifier parameters ['->' type_annotation] ':' suite

# Parameters for functions
parameters ::= '(' [parameter_list] ')'

# Parameter list
parameter_list ::= defparameter {',' defparameter} [',' [varargs [',' kwargs]]] |
                   varargs [',' kwargs] |
                   kwargs

# Default parameter
defparameter ::= identifier [':' type_annotation] ['=' expression]

# Variable arguments parameter
varargs ::= '*' [identifier [':' type_annotation]]

# Keyword arguments parameter
kwargs ::= '**' identifier [':' type_annotation]

# Class declaration
class_declaration ::= 'class' identifier ['(' [expression_list] ')'] ':' suite

# Type declaration (type alias)
type_declaration ::= 'type' identifier '=' type_expression
```

## 4. Simple Statements

```ebnf
# Expression statement
expression_statement ::= expression

# Assignment statement
assignment_statement ::= (target_list '=' )+ expression

target_list ::= target {',' target} [',']

target ::= identifier | attributeref | subscription | slicing

# Augmented assignment
augmented_assignment_statement ::= target augop expression

augop ::= '+=' | '-=' | '*=' | '/=' | '//=' | '%=' | '**=' |
         '>>=' | '<<=' | '&=' | '^=' | '|=' | '@='

# Pass statement
pass_statement ::= 'pass'

# Delete statement
del_statement ::= 'del' target_list

# Assert statement
assert_statement ::= 'assert' expression [',' expression]

# Global statement
global_statement ::= 'global' identifier {',' identifier}

# Nonlocal statement
nonlocal_statement ::= 'nonlocal' identifier {',' identifier}

# Return statement
return_statement ::= 'return' [expression_list]

# Break and continue statements
break_statement ::= 'break'
continue_statement ::= 'continue'

# Raise statement
raise_statement ::= 'raise' [expression ['from' expression]]

# Yield statement
yield_statement ::= yield_expression
```

## 5. Compound Statements

```ebnf
# If statement
if_statement ::= 'if' expression ':' suite {'elif' expression ':' suite} ['else' ':' suite]

# While statement
while_statement ::= 'while' expression ':' suite ['else' ':' suite]

# For statement
for_statement ::= 'for' target_list 'in' expression_list ':' suite ['else' ':' suite]

# With statement
with_statement ::= 'with' with_item {',' with_item} ':' suite

with_item ::= expression ['as' target]

# Try statement
try_statement ::= 'try' ':' suite
                  ((except_clause ':' suite) {(except_clause ':' suite)}
                  ['else' ':' suite] ['finally' ':' suite] |
                  'finally' ':' suite)

except_clause ::= 'except' [expression ['as' identifier]]

# Match statement
match_statement ::= 'match' expression ':' NEWLINE INDENT case_block {case_block} DEDENT

case_block ::= 'case' pattern [guard] ':' suite

guard ::= 'if' expression

# Async statements
async_for_statement ::= 'async' 'for' target_list 'in' expression_list ':' suite ['else' ':' suite]

async_with_statement ::= 'async' 'with' with_item {',' with_item} ':' suite
```

## 6. Expressions

```ebnf
# Expression hierarchy (in order of increasing precedence)
expression ::= conditional_expression

# Conditional expression (ternary)
conditional_expression ::= or_expression ['if' or_expression 'else' expression]

# Logical OR
or_expression ::= and_expression {'or' and_expression}

# Logical AND
and_expression ::= not_expression {'and' not_expression}

# Logical NOT
not_expression ::= ['not'] comparison

# Comparison operators
comparison ::= bitwise_or {comp_operator bitwise_or}

comp_operator ::= '<' | '>' | '==' | '>=' | '<=' | '!=' | 'in' | 'not' 'in' | 'is' | 'is' 'not'

# Bitwise OR
bitwise_or ::= bitwise_xor {'|' bitwise_xor}

# Bitwise XOR
bitwise_xor ::= bitwise_and {'^' bitwise_and}

# Bitwise AND
bitwise_and ::= shift_expr {'&' shift_expr}

# Shift operations
shift_expr ::= arith_expr {('<<'|'>>') arith_expr}

# Addition and subtraction
arith_expr ::= term {('+'|'-') term}

# Multiplication, division, etc.
term ::= factor {('*'|'/'|'%'|'//') factor}

# Factor (unary operations)
factor ::= ('+'|'-'|'~') factor | power

# Exponentiation
power ::= primary ['**' factor]

# Primary expressions
primary ::= atom | attributeref | subscription | slicing | call

# Atoms (basic elements)
atom ::= identifier | literal | enclosure

# Literals
literal ::= string_literal | bytes_literal | numeric_literal | boolean_literal | none_literal

# String literals
string_literal ::= [STRING | MULTILINE_STRING | FORMATTED_STRING | MULTILINE_FORMATTED_STRING | TEMPLATE_STRING | MULTILINE_TEMPLATE_STRING]

# Bytes literals
bytes_literal ::= [BYTES | MULTILINE_BYTES]

# Numeric literals
numeric_literal ::= [INT | FLOAT | IMAGINARY | HEX | BINARY | OCTAL]

# Boolean literals
boolean_literal ::= 'True' | 'False'

# None literal
none_literal ::= 'None'

# Enclosures (groupings, lists, dicts, sets)
enclosure ::= parenth_form | list_display | dict_display | set_display

# Parenthesized forms (including tuples)
parenth_form ::= '(' [expression_list] ')'

# Expression list (used in various contexts)
expression_list ::= expression {',' expression} [',']

# List displays
list_display ::= '[' [expression_list | comprehension] ']'

# Dictionary displays
dict_display ::= '{' [key_datum_list | dict_comprehension] '}'
key_datum_list ::= key_datum {',' key_datum} [',']
key_datum ::= expression ':' expression

# Set displays
set_display ::= '{' [expression_list | comprehension] '}'

# Comprehensions (list, dict, set)
comprehension ::= expression comp_for
dict_comprehension ::= key_datum comp_for
comp_for ::= ['async'] 'for' target_list 'in' or_expression [comp_iter]
comp_iter ::= comp_for | comp_if
comp_if ::= 'if' expression [comp_iter]

# Attribute reference, subscription, slicing
attributeref ::= primary '.' identifier
subscription ::= primary '[' expression_list ']'
slicing ::= primary '[' [expression] ':' [expression] [':' [expression]] ']'

# Function calls
call ::= primary '(' [argument_list] ')'
argument_list ::= positional_arguments [',' keyword_arguments] [',' starred_arguments]
                | keyword_arguments [',' starred_arguments]
                | starred_arguments
positional_arguments ::= expression {',' expression}
keyword_arguments ::= keyword_item {',' keyword_item}
keyword_item ::= identifier '=' expression
starred_arguments ::= '*' expression {',' '*' expression} [',' '**' expression]
                     | '**' expression

# Special expressions
lambda_expr ::= 'lambda' [parameter_list] ':' expression
await_expr ::= 'await' primary
yield_expr ::= 'yield' [expression_list | 'from' expression]
```

## 7. Patterns (for match statements)

```ebnf
# Pattern for match cases
pattern ::= literal_pattern
          | identifier_pattern
          | wildcard_pattern
          | sequence_pattern
          | mapping_pattern
          | class_pattern

# Literal pattern (matches exact values)
literal_pattern ::= numeric_literal | string_literal | boolean_literal | none_literal

# Identifier pattern (binds value to name)
identifier_pattern ::= identifier

# Wildcard pattern (matches anything)
wildcard_pattern ::= '_'

# Sequence pattern (lists, tuples)
sequence_pattern ::= '[' [pattern {',' pattern} [',']] [star_pattern] ']'

# Star pattern (captures remainder)
star_pattern ::= '*' identifier

# Mapping pattern (dictionaries)
mapping_pattern ::= '{' [key_pattern {',' key_pattern} [',']] [double_star_pattern] '}'

# Key pattern in mapping
key_pattern ::= expression ':' pattern

# Double star pattern (captures remainder of dict)
double_star_pattern ::= '**' identifier

# Class pattern (match by class with attribute patterns)
class_pattern ::= name '(' [pattern_arguments] ')'

# Pattern arguments
pattern_arguments ::= positional_patterns [',' keyword_patterns] [',']
                    | keyword_patterns [',']

positional_patterns ::= pattern {',' pattern}
keyword_patterns ::= keyword_pattern {',' keyword_pattern}
keyword_pattern ::= identifier '=' pattern
```

## 8. Type Annotations

```ebnf
# Type annotation (used in various contexts)
type_annotation ::= type_expression

# Type expression
type_expression ::= simple_type
                  | generic_type
                  | callable_type
                  | tuple_type
                  | union_type
                  | literal_type

# Simple type (a named type)
simple_type ::= identifier

# Generic type (e.g., List[int])
generic_type ::= simple_type '[' type_expression {',' type_expression} ']'

# Union type (e.g., int | str)
union_type ::= type_expression '|' type_expression {'|' type_expression}

# Callable type (e.g., Callable[[int, str], bool])
callable_type ::= 'Callable' '[' '[' [type_expression {',' type_expression}] ']' ',' type_expression ']'

# Tuple type (e.g., tuple[int, str])
tuple_type ::= 'tuple' '[' [type_expression {',' type_expression}] ']'

# Literal type (e.g., Literal["red", "green", "blue"])
literal_type ::= 'Literal' '[' literal {',' literal} ']'
```

## 9. Lexical Elements

```ebnf
# Identifiers
identifier ::= (letter | '_') (letter | digit | '_')*

# Keywords - these are reserved words that cannot be used as identifiers
keyword ::= 'False' | 'None' | 'True' | 'and' | 'as' | 'assert' | 'async' | 'await' |
            'break' | 'class' | 'continue' | 'def' | 'del' | 'elif' | 'else' |
            'except' | 'finally' | 'for' | 'from' | 'global' | 'if' | 'import' |
            'in' | 'is' | 'lambda' | 'nonlocal' | 'not' | 'or' | 'pass' |
            'raise' | 'return' | 'try' | 'while' | 'with' | 'yield' |
            'match' | 'case' | 'type'

# String literals
STRING ::= '"' {character - '"'} '"' | "'" {character - "'"} "'"

# Multiline string literals
MULTILINE_STRING ::= '"""' {character} '"""' | "'''" {character} "'''"

# Formatted string literals
FORMATTED_STRING ::= 'f' STRING
MULTILINE_FORMATTED_STRING ::= 'f' MULTILINE_STRING

# Template string literals (Typhon extension)
TEMPLATE_STRING ::= 't' STRING
MULTILINE_TEMPLATE_STRING ::= 't' MULTILINE_STRING

# Bytes literals
BYTES ::= 'b' STRING
MULTILINE_BYTES ::= 'b' MULTILINE_STRING

# Integer literals
INT ::= digit {digit | '_'}

# Hexadecimal literals
HEX ::= '0' ('x'|'X') hexdigit {hexdigit | '_'}

# Binary literals
BINARY ::= '0' ('b'|'B') ('0'|'1') {('0'|'1') | '_'}

# Octal literals
OCTAL ::= '0' ('o'|'O') octdigit {octdigit | '_'}

# Float literals
FLOAT ::= digit {digit | '_'} '.' {digit | '_'} [exponent]
          | digit {digit | '_'} exponent

exponent ::= ('e'|'E') ['+'|'-'] digit {digit | '_'}

# Imaginary literals
IMAGINARY ::= (INT | FLOAT) 'j'

# Operators
operator ::= '+' | '-' | '*' | '/' | '//' | '%' | '**' | '<<' | '>>' | '&' | '|' | '^' | '~' |
             ':=' | '<' | '>' | '<=' | '>=' | '==' | '!=' | '@'

# Delimiters
delimiter ::= '(' | ')' | '[' | ']' | '{' | '}' | ',' | ':' | '.' | ';' | '@' | '=' |
              '+=' | '-=' | '*=' | '/=' | '//=' | '%=' | '**=' |
              '<<=' | '>>=' | '&=' | '|=' | '^=' | '@='

# Indentation tokens (synthetic)
INDENT ::= <increased indentation>
DEDENT ::= <decreased indentation>
NEWLINE ::= <end of line>
```
