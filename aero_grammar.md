# Aero Language Specification - EBNF Grammar

## Introduction

This document formally specifies the core grammar of the Aero programming language using Extended Backus-Naur Form (EBNF). This grammar will serve as the definitive guide for implementing the lexer and parser components of the Aero compiler.

## Lexical Structure

### Identifiers

```ebnf
identifier = letter { letter | digit | "_" } ;
```

### Keywords

```ebnf
keyword = "fn" | "let" | "mut" | "if" | "else" | "while" | "for" | "return" | "struct" | "enum" | "trait" | "impl" | "pub" | "use" | "as" | "true" | "false" ;
```

### Literals

```ebnf
integer_literal = digit { digit } ;
float_literal = digit { digit } "." digit { digit } ;
string_literal = '"' { character - '"' } '"' ;
boolean_literal = "true" | "false" ;
```

### Operators

```ebnf
assignment_operator = "=" ;
arithmetic_operator = "+" | "-" | "*" | "/" | "%" ;
comparison_operator = "==" | "!=" | "<" | ">" | "<=" | ">=" ;
logical_operator = "&&" | "||" | "!" ;
```

## Syntactic Structure

### Program

```ebnf
program = { statement } ;
```

### Statements

```ebnf
statement = declaration | expression_statement | control_flow_statement ;
```

### Declarations

```ebnf
declaration = variable_declaration | function_declaration | struct_declaration | enum_declaration | trait_declaration | impl_declaration ;
```

### Expressions

```ebnf
expression = assignment_expression | binary_expression | unary_expression | literal_expression | identifier_expression | call_expression | member_access_expression | block_expression ;
```

## To be completed:

- Detailed EBNF for all language constructs (functions, control flow, types, etc.)
- Rules for whitespace and comments
- Error handling considerations




## Whitespace and Comments

### Whitespace

```ebnf
whitespace = ( " " | "\t" | "\n" | "\r" ) { " " | "\t" | "\n" | "\r" } ;
```

### Comments

```ebnf
line_comment = "//" { character - ( "\n" | "\r" ) } ( "\n" | "\r" ) ;
block_comment = "/*" { character - ( "*" "/" ) | block_comment } "*/" ;
comment = line_comment | block_comment ;
```

## Detailed Syntactic Structure

### Variable Declaration

```ebnf
variable_declaration = ( "let" | "mut" ) identifier [ ":" type ] [ "=" expression ] ";" ;
```

### Function Declaration

```ebnf
function_declaration = "fn" identifier "(" [ parameter_list ] ")" [ "->" type ] block_expression ;
parameter_list = parameter { "," parameter } ;
parameter = identifier ":" type ;
```

### Control Flow Statements

#### If Statement

```ebnf
if_statement = "if" expression block_expression [ "else" ( block_expression | if_statement ) ] ;
```

#### While Loop

```ebnf
while_loop = "while" expression block_expression ;
```

#### For Loop (Placeholder - detailed iteration protocol to be defined)

```ebnf
for_loop = "for" identifier "in" expression block_expression ;
```

#### Return Statement

```ebnf
return_statement = "return" [ expression ] ";" ;
```

### Block Expression

```ebnf
block_expression = "{" { statement } [ expression ] "}" ;
```

### Types

```ebnf
type = identifier [ "<" type_argument_list ">" ] ;
type_argument_list = type { "," type } ;
```

### Struct Declaration

```ebnf
struct_declaration = "struct" identifier [ "<" type_parameter_list ">" ] "{" { field_declaration } "}" ;
field_declaration = identifier ":" type ";" ;
type_parameter_list = identifier { "," identifier } ;
```

### Enum Declaration

```ebnf
enum_declaration = "enum" identifier [ "<" type_parameter_list ">" ] "{" { variant_declaration } "}" ;
variant_declaration = identifier [ "(" type_list ")" ] ;
type_list = type { "," type } ;
```

### Trait Declaration

```ebnf
trait_declaration = "trait" identifier [ "<" type_parameter_list ">" ] "{" { trait_function_signature } "}" ;
trait_function_signature = "fn" identifier "(" [ parameter_list ] ")" [ "->" type ] ";" ;
```

### Impl Declaration

```ebnf
impl_declaration = "impl" [ "<" type_parameter_list ">" ] identifier [ "for" type ] "{" { function_declaration } "}" ;
```

## Error Handling Considerations (High-Level)

- Lexical errors: Unrecognized characters, unclosed strings/comments.
- Syntactic errors: Mismatched parentheses/braces, missing semicolons, incorrect keyword usage.
- Semantic errors: Type mismatches, undefined variables, borrow checker violations (detailed in a separate document).



