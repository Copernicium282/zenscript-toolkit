; ZenScript — syntax highlighting queries.
;
; Grammar: ikexing-cn/tree-sitter-zenscript (vendored, completed)
;   grammar.js is the source of truth for node names; the queries below
;   reference every named node that benefits from a distinct capture.

; --- Comments --------------------------------------------------------------
(comment) @comment

; --- Imports ---------------------------------------------------------------
(import_declaration
  "import" @keyword.control.import
  (qualified_name
    name: (identifier) @namespace)
  (as) @keyword.control
  (identifier) @namespace)

; --- Classes ---------------------------------------------------------------
(class_declaration
  [
    "zenClass"
    "frigginClass"
  ] @keyword.control
  (identifier) @type)

(constructor_declaration
  [
    "zenConstructor"
    "frigginConstructor"
  ] @keyword.control)

; --- Functions -------------------------------------------------------------
(function_declaration
  "function" @keyword.control
  (identifier) @function
  (as) @keyword.control
  (return_type) @type)

(expand_function_declaration
  "$expand" @keyword.control
  (identifier) @function)

(lambda_expression
  "function" @keyword.control)

; --- Statements / control flow ---------------------------------------------
(if_statement
  [
    "if"
    "else"
  ] @keyword.control.conditional)

(foreach_statement
  [
    "for"
    "in"
  ] @keyword.control)

(while_statement
  "while" @keyword.control)

[
  "return"
  "break"
  "continue"
] @keyword.control.return

; --- Version preprocessor-style statement ----------------------------------
(version_statement
  "version" @keyword.control)

; --- Variable declarations -------------------------------------------------
(variable_declaration
  [
    "var"
    "val"
    "static"
    "global"
  ] @keyword.control
  (identifier) @variable
  (as) @keyword.control
  (initializer) @variable)

; --- Function parameters ---------------------------------------------------
(formal_parameter
  (identifier) @variable.parameter
  (as) @keyword.control)

; --- Types -----------------------------------------------------------------
(class_type
  (identifier) @type)

(qualified_name
  (identifier) @type)

(function_type
  "function" @keyword.control.type
  (return_type) @type)

(list_type
  (_) @type)

(array_type
  (_) @type)

(map_type
  (key: (_) @type)
  (value: (_) @type))

(primitive_type) @type.builtin

; --- Expressions -----------------------------------------------------------

; Atoms
(number_literal) @number
(string_literal) @string
(string_fragment) @string
(escape_sequence) @string.escape
(boolean_literal) @constant.builtin.boolean
(null_literal) @constant.builtin

; Containers
(array_literal
  "[" @punctuation.bracket
  "]" @punctuation.bracket)

(map_literal
  "{" @punctuation.bracket
  "}" @punctuation.bracket)

(map_entry
  ":" @punctuation.delimiter)

; Bracket handlers — the `<item:minecraft:diamond>` syntax
(bracket_handler
  "<" @punctuation.special
  ">" @punctuation.special)

; Member access
(member_access_expression
  "." @punctuation.delimiter
  (property) @property)

; Function calls
(call_expression
  (function) @function)

; Indexing
(index_expression
  "[" @punctuation.bracket
  "]" @punctuation.bracket)

; Casts
(type_cast_expression
  (value) @variable
  "as" @keyword.control
  (type) @type)

(instanceof_expression
  (value) @variable
  "instanceof" @keyword.control
  (type) @type)

; Range
(range_expression
  (start) @variable
  [
    ".."
  ] @operator
  (end) @variable)

; Ternary
(ternary_expression
  [
    "?"
    ":"
  ] @keyword.control.ternary)

; Assignment
(assignment_expression
  (operator) @operator)

; Unary
(unary_expression
  (operator) @operator)

; Binary
(binary_expression
  (operator) @operator)

; --- Identifiers (catch-all) -----------------------------------------------
; Painted last so the more specific captures above take priority.
(identifier) @variable

; --- Brackets / punctuation ------------------------------------------------
[
  "{"
  "}"
  "["
  "]"
  "("
  ")"
] @punctuation.bracket

[
  ";"
  ","
  "."
] @punctuation.delimiter
