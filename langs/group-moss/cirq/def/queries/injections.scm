; Cirq tree-sitter language injections
; ============================================================
;
; Inject embedded-language highlighting / parsing into the body of
; `code "lang" { ... }` blocks. The language string inside the
; quotes selects which grammar to use for the body.
;
; Editors look up the injected grammar by the exact captured text,
; so well-known short names are mapped to their canonical grammar
; name below. Anything not aliased falls through to the generic
; rule at the bottom, which strips the surrounding quotes and uses
; the string verbatim — so `code "rust" {}` or `code "python" {}`
; just work.

; ── Aliases ──────────────────────────────────────────────────

((code_decl
  language: (string_literal) @_lang
  body: (code_body) @injection.content)
 (#any-of? @_lang "\"js\"" "\"jsx\"")
 (#set! injection.language "javascript")
 (#set! injection.combined))

((code_decl
  language: (string_literal) @_lang
  body: (code_body) @injection.content)
 (#any-of? @_lang "\"ts\"" "\"tsx\"")
 (#set! injection.language "typescript")
 (#set! injection.combined))

((code_decl
  language: (string_literal) @_lang
  body: (code_body) @injection.content)
 (#eq? @_lang "\"py\"")
 (#set! injection.language "python")
 (#set! injection.combined))

((code_decl
  language: (string_literal) @_lang
  body: (code_body) @injection.content)
 (#eq? @_lang "\"rs\"")
 (#set! injection.language "rust")
 (#set! injection.combined))

((code_decl
  language: (string_literal) @_lang
  body: (code_body) @injection.content)
 (#eq? @_lang "\"sh\"")
 (#set! injection.language "bash")
 (#set! injection.combined))

((code_decl
  language: (string_literal) @_lang
  body: (code_body) @injection.content)
 (#eq? @_lang "\"control\"")
 (#set! injection.language "cirq-control")
 (#set! injection.combined))

((code_decl
  language: (string_literal) @_lang
  body: (code_body) @injection.content)
 (#eq? @_lang "\"md\"")
 (#set! injection.language "markdown")
 (#set! injection.combined))

; ── Generic fallback ─────────────────────────────────────────
; Strip the surrounding quotes from the string literal and use
; the inner text as the injection language. Excluded short names
; are handled by the alias rules above.

((code_decl
  language: (string_literal) @injection.language
  body: (code_body) @injection.content)
 (#not-any-of? @injection.language
   "\"js\"" "\"jsx\"" "\"ts\"" "\"tsx\""
   "\"py\"" "\"rs\"" "\"sh\"" "\"md\"" "\"control\"")
 (#offset! @injection.language 0 1 0 -1)
 (#set! injection.combined))
