# Aero Language Specification - EBNF Grammar

## Introduction

This document formally specifies the core grammar of the Aero programming language using Extended Backus-Naur Form (EBNF). This grammar will serve as the definitive guide for implementing the lexer and parser components of the Aero compiler.

## Lexical Analysis

Lexical analysis is the first phase of the Aero compiler, where the source code is scanned and broken down into a sequence of tokens. Tokens are the smallest meaningful units of the language.

### Token Types

The primary token types in Aero are:
- Identifiers
- Keywords
- Literals (Integer, Float, String, Boolean)
- Operators
- Delimiters (Parentheses, Braces, Brackets, etc.)
- Comments
- Whitespace

### Identifiers

```ebnf
identifier = letter { letter | digit | "_" } ;
```

### Keywords

```ebnf
keyword = "fn" | "let" | "mut" | "if" | "else" | "while" | "for" | "return" | "struct" | "enum" | "trait" | "impl" | "pub" | "use" | "as" | "true" | "false" | "mod" | "const" | "type" | "loop" | "break" | "continue" | "print!" | "println!" ;
```

### Literals

```ebnf
integer_literal = digit { digit } ;
float_literal = digit { digit } "." digit { digit } ;
string_literal = '"' { character - '"' | '\\"' } '"' ; // Allow escaped quotes
boolean_literal = "true" | "false" ;
char_literal = "'" ( character - "'" - "\\" | "\\" ( "n" | "r" | "t" | "\\" | "'" | '"' | "0" | "x" hex_digit hex_digit ) ) "'" ;
hex_digit = digit | "a".."f" | "A".."F" ;
```

### Operators

```ebnf
assignment_operator = "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" ;
arithmetic_operator = "+" | "-" | "*" | "/" | "%" ;
comparison_operator = "==" | "!=" | "<" | ">" | "<=" | ">=" ;
logical_operator = "&&" | "||" ; // "!" is unary
unary_operator = "!" | "-" | "&" [ "mut" ] | "*" ; // Includes dereference, borrow, logical NOT, negation
bitwise_operator = "&" | "|" | "^" | "<<" | ">>" ;
```

### Delimiters

```ebnf
delimiter = "(" | ")" | "{" | "}" | "[" | "]" | "," | ":" | ";" | "->" | "." ;
```

### Whitespace and Comments

Whitespace and comments are generally ignored by the parser, except where they serve to separate tokens.

#### Whitespace

```ebnf
whitespace = ( " " | "\t" | "\n" | "\r" ) { " " | "\t" | "\n" | "\r" } ;
```

#### Comments

```ebnf
line_comment = "//" { character - ( "\n" | "\r" ) } ( "\n" | "\r" ) ;
block_comment = "/*" { character - ( "*" "/" ) | block_comment } "*/" ;
comment = line_comment | block_comment ;
```

## Syntactic Grammar

This section details the EBNF rules for the syntactic structure of the Aero language.

### Program Structure

Aero programs are organized into modules.

```ebnf
program = { module_item } ;
module_item = function_declaration
            | struct_declaration
            | enum_declaration
            | trait_declaration
            | impl_declaration
            | module_declaration
            | import_statement
            | constant_declaration
            | type_alias_declaration ;
```

### Module and Import Statements

Modules allow for organizing code into namespaces and controlling visibility.

```ebnf
module_declaration = [ "pub" ] "mod" identifier ( ";" | "{" { module_item } "}" ) ;
import_statement = [ "pub" ] "use" import_path [ "as" identifier ] ";" ;
import_path = identifier { "::" identifier } [ "::" ( "*" | "{" import_list "}" ) ] ;
import_list = import_path_segment { "," import_path_segment } [ "," ] ;
import_path_segment = identifier [ "as" identifier ] ;
```

### Statements

```ebnf
statement = local_declaration | expression_statement | control_flow_statement ;
// Note: module_declaration, import_statement, and major declarations (functions, structs, etc.)
// are module_items, not general statements allowed directly within blocks like function bodies.
// Local declarations are e.g. variable declarations.
```

### Declarations

This section now distinguishes between top-level (module item) declarations and local declarations.

#### Module-Level Declarations
The following are `module_item`s:
- `function_declaration`
- `struct_declaration`
- `enum_declaration`
- `trait_declaration`
- `impl_declaration`
- `module_declaration` (see above)
- `import_statement` (see above)
- `constant_declaration`
- `type_alias_declaration`

```ebnf
constant_declaration = [ "pub" ] "let" "const" identifier ":" type "=" expression ";" ;
type_alias_declaration = [ "pub" ] "type" identifier [ "<" type_parameter_list ">" ] "=" type ";" ;
```

#### Local Declarations
```ebnf
local_declaration = variable_declaration ; // For now, only 'let' and 'mut' are local.
// Future: local item declarations (e.g. functions inside functions) if supported.
declaration = variable_declaration | constant_declaration | type_alias_declaration; // Retaining a general 'declaration' for local items. Module items are listed in program_structure.
```

### Expressions

The following EBNF for expressions defines operator precedence and associativity.
Lower precedence operators are defined first and use higher precedence operators as operands.

```ebnf
expression = assignment_expression ;

assignment_expression = conditional_expression [ assign_op conditional_expression ] ; // Right-associative, simplified: no multiple assignments like a=b=c without parens
assign_op = "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" ;

conditional_expression = logical_or_expression [ "?" expression ":" conditional_expression ] ; // Ternary conditional

logical_or_expression = logical_and_expression { "||" logical_and_expression } ;
logical_and_expression = bitwise_or_expression { "&&" bitwise_or_expression } ;

bitwise_or_expression = bitwise_xor_expression { "|" bitwise_xor_expression } ;
bitwise_xor_expression = bitwise_and_expression { "^" bitwise_and_expression } ;
bitwise_and_expression = equality_expression { "&" equality_expression } ;

equality_expression = comparison_expression { ( "==" | "!=" ) comparison_expression } ;
comparison_expression = shift_expression { ( "<" | ">" | "<=" | ">=" ) shift_expression } ;

shift_expression = additive_expression { ( "<<" | ">>" ) additive_expression } ;
additive_expression = multiplicative_expression { ( "+" | "-" ) multiplicative_expression } ;
multiplicative_expression = cast_expression { ( "*" | "/" | "%" ) cast_expression } ; // Changed from unary_expression to cast_expression

cast_expression = unary_expression [ "as" type ] ; // e.g. x as i64

unary_expression = ( "!" | "-" ) unary_expression   // Prefix logical NOT, arithmetic negation
                 | "&" [ "mut" ] unary_expression  // Borrow
                 | "*" unary_expression            // Dereference
                 | primary_expression ;

primary_expression = literal
                   | identifier_path                 // General path: my_var, my_mod::MY_CONST, MyEnum::Variant
                   | "(" expression ")"              // Grouped expression
                   | block_expression
                   | if_expression                   // if can be an expression
                   | loop_expression                 // loop can be an expression (value from break)
                   | while_expression                // while can be an expression (value from break)
                   | for_expression                  // for can be an expression (value from break)
                   | array_literal
                   | tuple_literal
                   | call_expression
                   | index_expression                // array_or_slice[index]
                   | field_access_expression         // struct_instance.field
                   | tuple_index_expression          // tuple_instance.0
                   | print_macro                     // print! macro
                   | println_macro                   // println! macro
                   | "break" [ loop_label_ref ] [ expression ] // break as an expression
                   | "return" [ expression ] ;       // return as an expression (diverges)

literal = integer_literal | float_literal | string_literal | boolean_literal | char_literal ;

identifier_path = identifier { "::" identifier } ; // Used for variables, constants, enum variants, static functions. Disambiguated from `path_type` by parser context.

call_expression = primary_expression "(" [ argument_list ] ")" ;
argument_list = expression { "," expression } [ "," ] ;

index_expression = primary_expression "[" expression "]" ;
field_access_expression = primary_expression "." identifier ;
tuple_index_expression = primary_expression "." integer_literal ; // e.g. my_tuple.0

array_literal = "[" [ expression { "," expression } [ "," ] ] "]" ;
tuple_literal = "(" [ expression { "," expression } [ "," ] ] ")" ;

// I/O macro expressions
print_macro = "print!" "(" string_literal [ "," argument_list ] ")" ;
println_macro = "println!" "(" string_literal [ "," argument_list ] ")" ;

// Control flow constructs as expressions
loop_expression = [ loop_label ] "loop" block_expression ;
while_expression = [ loop_label ] "while" expression block_expression ; // Value from break
for_expression = [ loop_label ] "for" pattern "in" expression block_expression ; // Value from break
if_expression = "if" expression block_expression [ "else" ( block_expression | if_expression ) ] ;

```

### Variable Declaration

```ebnf
variable_declaration = "let" [ "mut" ] identifier [ ":" type ] [ "=" expression ] ";" ;
```

### Function Declaration
```ebnf
function_declaration = [ "pub" ] "fn" identifier [ "<" type_parameter_list ">" ] "(" [ parameter_list ] ")" [ "->" type ] ( block_expression | ";" ) ;
parameter_list = parameter { "," parameter } [ "," ] ;
parameter = [ "mut" ] identifier ":" type ;
type_parameter_list = identifier { "," identifier } [ "," ] ;
```

### Control Flow Statements and Expressions
Many control flow constructs can be used as statements or expressions. The statement forms are defined here. Their expression forms are included in `primary_expression`.

```ebnf
loop_label = identifier ; // Used before ':', e.g. my_loop:
loop_label_ref = identifier ; // Used after break/continue

if_statement = "if" expression block_expression [ "else" ( block_expression | if_statement ) ] ;
loop_statement = [ loop_label ":" ] "loop" block_expression ;
while_statement = [ loop_label ":" ] "while" expression block_expression ;
for_statement = [ loop_label ":" ] "for" pattern "in" expression block_expression ;

pattern = identifier // Simple pattern for variable binding
        | tuple_pattern
        | struct_pattern ; // More complex patterns (e.g., with `_`, `..`, `mut`, literals) will be detailed further.

tuple_pattern = "(" [ pattern { "," pattern } [ "," ] ] ")" ;
struct_pattern = identifier_path "{" [ field_pattern { "," field_pattern } [ "," ] ] "}" ; // e.g. MyStruct { field1: p1, field2 }
field_pattern = identifier [ ":" pattern ] ;

// Note: if_expression, loop_expression, while_expression, for_expression are defined under `primary_expression`.
```

#### Break, Continue, and Return Statements

```ebnf
break_statement = "break" [ loop_label_ref ] [ expression ] ";" ;
continue_statement = "continue" [ loop_label_ref ] ";" ;
return_statement = "return" [ expression ] ";" ;
```

### Block Expression
```ebnf
block_expression = "{" { statement } [ expression ] "}" ; // The last expression is the value of the block
```

### Expression Statement
```ebnf
expression_statement = expression ";" ;
```

### Types

```ebnf
type = path_type | tuple_type | array_type | reference_type | pointer_type | function_type ;

path_type = identifier { "::" identifier } [ "<" type_argument_list ">" ] ; // e.g. simple_type, module::Type, or Type<T, U>
type_argument_list = type { "," type } [ "," ] ; // Allow trailing comma

tuple_type = "(" [ type { "," type } [ "," ] ] ")" ; // e.g. (i32, String) or () for unit type
array_type = "[" type ( ";" expression )? "]" ; // e.g. [i32; 5] for array, [i32] for slice. Expression is const size.

reference_type = "&" [ "mut" ] type ; // e.g. &MyType, &mut AnotherType
pointer_type = "*" ( "const" | "mut" ) type ; // e.g. *const u8, *mut Data

function_type = [ "unsafe" ] "fn" "(" [ type_list_for_function_type ] ")" [ "->" type ] ; // Type of a function pointer/closure
type_list_for_function_type = type { "," type } [ "," ] ; // Separate from parameter_list as it doesn't have names
```

### Struct Declaration

```ebnf
struct_declaration = [ "pub" ] "struct" identifier [ "<" type_parameter_list ">" ] ( struct_fields_block | tuple_fields | ";" ) ;
struct_fields_block = "{" { [ "pub" ] field_declaration } "}" ; // For structs with named fields
tuple_fields = "(" { [ "pub" ] type } [ "," ] ")" ; // For tuple-like structs, e.g. struct Point(i32, i32);
field_declaration = identifier ":" type ";" ;
// type_parameter_list already defined under function_declaration
```

### Enum Declaration

```ebnf
enum_declaration = [ "pub" ] "enum" identifier [ "<" type_parameter_list ">" ] "{" { enum_variant } "}" ;
enum_variant = identifier [ enum_variant_kind ] [ "," ] ;
enum_variant_kind = tuple_variant_fields | struct_variant_fields ;
tuple_variant_fields = "(" type { "," type } [ "," ] ")" ; // e.g. MyEnum::TupleVariant(i32, String)
struct_variant_fields = "{" { [ "pub" ] field_declaration } "}" ; // e.g. MyEnum::StructVariant { id: i32, value: String }
// type_list is effectively covered by tuple_variant_fields
```

### Trait Declaration

```ebnf
trait_declaration = [ "pub" ] [ "unsafe" ] "trait" identifier [ "<" type_parameter_list ">" ] [ ":" type_bounds ] "{" { associated_item } "}" ;
type_bounds = path_type { "+" path_type } ; // Supertraits, e.g. trait MyTrait: Debug + Clone
associated_item = trait_function_signature | trait_type_alias | trait_constant_declaration ;

trait_function_signature = "fn" identifier [ "<" type_parameter_list ">" ] "(" [ parameter_list ] ")" [ "->" type ] ";" ;
trait_type_alias = "type" identifier [ "<" type_parameter_list ">" ] [ ":" type_bounds ] [ "=" type ] ";" ; // RHS type is optional in trait
trait_constant_declaration = "const" identifier ":" type ";" ; // Value is not provided in trait
```

### Impl Declaration

```ebnf
impl_declaration = [ "unsafe" ] "impl" [ "<" type_parameter_list ">" ] type [ "for" path_type ] "{" { impl_item } "}" ;
// First 'type' is the implementing type (e.g. MyStruct<T>).
// 'path_type' after 'for' is the trait being implemented (e.g. MyTrait<U>).
// If 'for path_type' is omitted, it's an inherent impl.
impl_item = function_declaration | constant_declaration | type_alias_declaration ; // Items allowed in an impl block (must match associated items if for a trait)
```

## Error Handling Strategies

Robust error handling is crucial for a good compiler user experience. When the parser encounters code that doesn't conform to the grammar, it needs to report errors effectively and, if possible, recover to find more errors.

### Common Error Types

#### Lexical Errors
- **Unrecognized Characters:** Reporting the invalid character and its location.
- **Unclosed Strings/Comments:** Reporting the start of the unclosed literal or comment and the location where it was expected to close.
- **Invalid Number/String Formatting:** e.g., `0xG` or ` "abcde`.
- **Invalid Character Literal:** e.g. `''` or `'ab'`.

#### Syntactic Errors
- **Unexpected Token:** The parser encounters a token that is not valid at the current position according to the grammar rules. For example, finding an `else` without a preceding `if`, or a `break` outside a loop.
- **Missing Token:** The parser expects a certain token (e.g., a semicolon, a closing parenthesis, a type annotation, an operator) but finds a different token or the end of the input.
- **Mismatched Delimiters:** Unbalanced parentheses `()`, braces `{}`, or brackets `[]`.
- **Incomplete Constructs:** e.g., `let x = ;` or `fn my_func( ;` or `if condition`.
- **Incorrect Keyword Usage:** Using a keyword in an invalid context, e.g. `return` at the top level of a module, or `mut` in a type position.
- **Expression Errors:** Malformed expressions, operator precedence issues misunderstood by the user, e.g. `a + b * c` parsed differently than expected if rules are not clear.

### Error Recovery Techniques

1.  **Panic Mode:**
    *   **Description:** When an error is detected, the parser discards tokens until it finds a "synchronization token" – a token that is likely to mark the beginning of a new, valid construct (e.g., a semicolon `;`, a keyword like `fn`, `let`, `struct`, `if`, or a closing brace `}`). After finding such a token, the parser attempts to resume parsing from that point.
    *   **Pros:** Relatively simple to implement. Can often skip over large problematic sections and find subsequent unrelated errors in the same file.
    *   **Cons:** Can discard a significant amount of input, potentially missing multiple errors within the discarded section. Error messages might be less precise if the parser is far from the actual point of error. Synchronization points need to be chosen carefully.

2.  **Phrase-Level Recovery:**
    *   **Description:** The parser has specific error-handling routines for particular grammatical phrases. For example, if a comma is missing in a parameter list, the parser might insert the comma (internally) and report an error, then continue parsing the list. Or if it sees `if condition { ... } else if condition { ... }` it might recover better than basic panic mode by expecting a block or another `if`.
    *   **Pros:** More precise error messages and can often recover more gracefully, leading to fewer cascaded errors.
    *   **Cons:** More complex to implement as it requires anticipating common error patterns for many different grammar rules.

3.  **Error Productions:**
    *   **Description:** The grammar is augmented with special "error productions" that explicitly match common erroneous constructs. When an error production is matched, an error is reported, but parsing can continue as if a valid (though erroneous) construct was found. For example, an error production might match `expression ; expression` within a block where a statement is expected, to catch missing semicolons between expressions used as statements.
    *   **Pros:** Can provide very specific error messages for known patterns of mistakes. Allows the parser to build a partial AST even in the presence of some errors.
    *   **Cons:** Can significantly complicate the grammar and the parser generation process. Requires identifying common error patterns beforehand. Care must be taken to avoid ambiguities with correct grammar.

4.  **Contextual Recovery / Heuristics:**
    *   **Description:** The parser uses more context (e.g., what kind of construct it is currently trying to parse, what tokens are expected next, indentation) to make a more intelligent decision about how to recover. This might involve inserting a missing token, deleting an unexpected token, or replacing a token.
    *   **Pros:** Can lead to very good recovery and highly relevant error messages.
    *   **Cons:** Can be very complex to design and implement correctly. Heuristics might not always guess the programmer's intent correctly and could lead to cascading errors if the guess is wrong.

Aero's parser will likely start with a robust panic mode strategy, focusing on good synchronization tokens (e.g., beginning of statements, declarations, closing delimiters). As the compiler matures, phrase-level recovery and specific error productions for very common mistakes could be incorporated. The goal is to provide clear, actionable error messages that help the developer quickly identify and fix issues in their code.

Semantic errors (type mismatches, undefined variables/functions, arity errors, trait bound violations, borrow checker violations) are typically handled in later phases of compilation (semantic analysis, type checking) after a syntactically valid (or partially valid, with error productions) Abstract Syntax Tree (AST) has been constructed.