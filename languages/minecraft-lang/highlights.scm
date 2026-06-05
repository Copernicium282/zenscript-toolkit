; Minecraft .lang files — syntax highlighting queries.
;
; Grammar: tree-sitter-grammars/tree-sitter-properties @ 6310671b24d4e04b803577b1c675d765cbd5773b
;
; The properties grammar is generic, so the captures below map the grammar's
; generic `key` / `value` nodes onto the conventional Minecraft .lang shapes
; the zz5840.minecraft-lang-colorizer VS Code extension highlights. Each
; capture name is chosen so the color under the 2026 Dark theme matches
; what VS Code renders.
;
; --- Comments --------------------------------------------------------------
(comment) @comment

; --- Keys ------------------------------------------------------------------
; VS Code: `entity.other.attribute-name.keys.minecraft-lang` → falls back to
; `entity.other.attribute-name` (#9cdcfe) → @attribute.
(key) @attribute

; --- Values ----------------------------------------------------------------
; VS Code: `entity.name.function.values.minecraft-lang` → falls back to
; `entity.name.function` (#d2a8ff) → @function.
(value) @function

; Escape sequences within values (\\n, \\t, \\u00A0, …).
(value (escape) @string.escape)

; Minecraft format codes (§0-§f, §k-§o, §r, §x…) sit inline inside the
; value text. The properties grammar does not tokenize them individually,
; so they cannot be given a token-level capture here. The closest
; approximation is to mark the whole value as `@string.special` when it
; contains a §, which gives the user a visual cue that formatting is
; embedded even if each code letter is not separately colored.
((value) @string.special
 (#match? @string.special "§[0-9a-frk-oA-FK-OR]"))

; --- Substitution: ${other.key} or ${OTHER_KEY::secret} --------------------
; VS Code: `variable.parameter.values.minecraft-lang` (the substituted
; name) → falls back to `variable.parameter.function` (#c9d1d9) →
; @variable.parameter. Brackets / `::` are not separately colored in
; VS Code, but giving them a dedicated capture helps the user see the
; substitution structure at a glance.
(substitution
  (key) @variable.parameter)

(substitution
  [
    "${"
    "}"
    "::"
  ] @punctuation.special)

; The `::secret` part is `variable.parameter` in dark-2026 dark; the
; VS Code colorizer by zz5840 also treats it as parameter-like.
(substitution
  (secret) @variable.parameter)

; --- Array-style index on a key (e.g. item.modid.sword[0]) ----------------
; VS Code: the bracketed number is part of the key shape, no specific
; scope; @number gives the conventional Zed color (#b5cea8) and matches
; the operator-like color most other editors use here.
(key (index) @number)
((key (index) @number)
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
