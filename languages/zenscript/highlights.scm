; ZenScript — syntax highlighting queries.
;
; Grammar: Copernicium282/tree-sitter-zenscript (fork of ikexing-cn)
; VS Code TextMate grammar: yesterday17.zenscript-0.2.10
;
; This maps tree-sitter nodes to capture names that reproduce the VS Code
; 2026 Dark theme colors. Last matching pattern wins; specific patterns go
; AFTER catch-all to override it.

; --- Identifiers (catch-all) -----------------------------------------------
(identifier) @variable

; --- Comments --------------------------------------------------------------
(comment) @comment

; --- Imports ---------------------------------------------------------------
; `import` is `keyword.other.import.zenscript` (VS Code) → falls back to
; `keyword` in 2026 Dark → @keyword.import (#ff7b72).
(import_declaration
  "import" @keyword.import)

(import_declaration
  (qualified_name
    name: (identifier) @namespace))

(import_declaration
  (qualified_name
    scope: (identifier) @namespace))

(import_declaration
  (qualified_name
    scope: (qualified_name
      name: (identifier) @namespace)))

(import_declaration
  (as) @keyword.operator)

; --- Version preprocessor-style statement ----------------------------------
; `version` is part of `meta.preprocessor.zenscript` (VS Code) → falls
; back to `meta.preprocessor` → @preproc (#569cd6).
(version_statement
  "version" @preproc)

; --- Classes ---------------------------------------------------------------
; `zenClass` / `frigginClass` are `keyword.other.class.zenscript` (VS Code)
; → falls back to `keyword` → @keyword.declaration (#ff7b72).
(class_declaration
  keyword: [
    "zenClass"
    "frigginClass"
  ] @keyword.declaration
  name: (identifier) @type)

(constructor_declaration
  keyword: [
    "zenConstructor"
    "frigginConstructor"
  ] @keyword.declaration)

; --- Functions -------------------------------------------------------------
; `function` is `storage.type.function.zenscript` (VS Code) → falls back to
; `storage` → @keyword.declaration (#ff7b72).
(function_declaration
  "function" @keyword.declaration
  name: (identifier) @function
  (as) @keyword.operator
  return_type: (_) @type)

; Highlight `static` on a function declaration separately because
; mixing it into the pattern above makes the query "impossible"
; (optional siblings that the parser cannot order).
(function_declaration
  (static) @keyword.declaration)

(expand_function_declaration
  "$expand" @keyword.declaration
  (identifier) @function)

(lambda_expression
  "function" @keyword.declaration)

; --- Statements / control flow ---------------------------------------------
; `if`/`else` are `keyword.control.conditional.zenscript` (VS Code) → falls
; back to `keyword.control` → @keyword.control.conditional (#c586c0).
(if_statement
  ["if" "else"] @keyword.control.conditional)

(foreach_statement
  ["for" "in"] @keyword.control)

(while_statement
  "while" @keyword.control)

["return" "break" "continue"] @keyword.control.return

; --- Variable declarations -------------------------------------------------
; `var`/`val` are `storage.type.var` (VS Code) → falls back to `storage`
; → @keyword.declaration (#ff7b72). `static`/`global`/ are
; `storage.modifier.{static,global}` → `storage.modifier` (no language-
; specific override) → @keyword.declaration (#ff7b72) — VS Code would
; render these #569cd6, but the closest semantically-correct capture
; here maps to the same generic-storage color the rest of the prefix
; group gets.
(variable_declaration
  prefix: ["var" "val"] @keyword.declaration
  name: (identifier) @variable)

(variable_declaration
  (static) @keyword.declaration)

(variable_declaration
  (global) @keyword.declaration)

(variable_declaration
  (as) @keyword.operator)

(variable_declaration
  initializer: (_) @variable)

; --- Function parameters ---------------------------------------------------
(formal_parameter
  (identifier) @variable.parameter
  (as) @keyword.operator)

; --- Types -----------------------------------------------------------------
(class_type
  (identifier) @type)

(qualified_name
  (identifier) @type)

(function_type
  "function" @keyword.control.type
  return_type: (_) @type)

(list_type
  (_) @type)

(array_type
  (_) @type)

; `map_type` has overlapping `key`/`value` fields (both
; `_type_literal`); a query that names both fields with `_` is rejected
; by the tree-sitter query compiler as impossible. Anchor the two
; captures to the order the grammar emits them in.
(map_type
  (_) @type
  .
  (_) @type)

; Primitive types (any, bool, byte, …) are `constant.other.type.zenscript`
; (VS Code) → falls back to `constant` → @type.builtin (#79c0ff).
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

; Bracket handlers — `<item:minecraft:diamond>` syntax. The angle brackets
; themselves are `variable.language.brackethandler.zenscript` (VS Code) →
; falls back to `variable.language` → @punctuation.special. The interior
; is `variable.parameter.brackethandler.zenscript` → falls back to
; `variable.parameter` → @variable.parameter.
(bracket_handler
  "<" @punctuation.special
  ">" @punctuation.special)

; Member access
(member_access_expression
  "." @punctuation.delimiter
  property: (_) @property)

; Function calls
(call_expression
  function: (_) @function)

; Indexing
(index_expression
  "[" @punctuation.bracket
  "]" @punctuation.bracket)

; Casts
(type_cast_expression
  value: (_) @variable
  "as" @keyword.operator
  type: (_) @type)

(instanceof_expression
  value: (_) @variable
  "instanceof" @keyword.operator
  type: (_) @type)

; Range
(range_expression
  start: (_) @variable
  operator: [".."] @operator
  end: (_) @variable)

; Ternary
(ternary_expression
  ["?" ":"] @keyword.control.ternary)

; Assignment
(assignment_expression
  operator: ["%=" "&=" "*=" "+=" "-=" "/=" "=" "^=" "|=" "~="] @operator)

; Unary
(unary_expression
  operator: ["!" "-"] @operator)

; --- Brackets / punctuation ------------------------------------------------
["{" "}" "[" "]" "(" ")"] @punctuation.bracket

[";" "," "."] @punctuation.delimiter