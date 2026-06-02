; Minecraft .lang files — syntax highlighting queries.
;
; Grammar: tree-sitter-grammars/tree-sitter-properties @ 6310671b24d4e04b803577b1c675d765cbd5773b
;
; The properties grammar is generic, so the captures below map the grammar's
; generic `key` / `value` nodes onto the conventional Minecraft .lang shapes:
; namespace dot separated keys, comments, and embedded Minecraft format
; codes (§0-§f, §r, §k-§o) inside string values. The Minecraft format codes
; themselves are not given dedicated captures by the underlying grammar (its
; `_char` node is a flat per-character match), so they inherit the value's
; @string styling.

; --- Comments --------------------------------------------------------------
(comment) @comment

; --- Keys ------------------------------------------------------------------
(key) @property

; --- Values ----------------------------------------------------------------
(value) @string

; Escape sequences within values (\\n, \\t, \\u00A0, …).
(value (escape) @string.escape)

; Substitution: ${other.key} or ${OTHER_KEY::secret}.
(substitution
  (key) @constant
  (#match? @constant "^[A-Z0-9_.]+$"))

(substitution
  [
    "${"
    "}"
    ":"
  ] @punctuation.special
  "::" @punctuation.special)

; Array-style index on a key (e.g. item.modid.sword[0]).
(index) @number
((index) @number
  (#match? @number "^[0-9]+$"))

; --- Operators and delimiters ---------------------------------------------
(property
  [
    "="
    ":"
  ] @operator)

[
  "."
  "\\"
] @punctuation.delimiter

[
  "["
  "]"
] @punctuation.bracket
