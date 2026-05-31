; Cirq tree-sitter highlights
; ============================================================

; ── Keywords ─────────────────────────────────────────────────

[
  "circuit"
  "module"
  "port"
  "param"
  "let"
  "model"
  "analysis"
  "import"
  "as"
  "global"
  "sweep"
  "step"
  "code"
] @keyword

; ── Code blocks ──────────────────────────────────────────────
; The language tag is conceptually a type-like marker rather than
; a string literal, so highlight it distinctly. The body is left
; unhighlighted here — injections.scm hands it off to the embedded
; language's own highlight queries.

(code_decl
  language: (string_literal) @string.special)

; Port directions
(port_direction) @keyword

; ── Literals ─────────────────────────────────────────────────

(number_literal) @number
(string_literal) @string
(boolean_literal) @boolean

; gnd is a built-in constant
(gnd) @constant.builtin

; ── Comments ─────────────────────────────────────────────────

(line_comment) @comment
(block_comment) @comment

; ── Operators ────────────────────────────────────────────────

(binary_expression
  operator: _ @operator)

(unary_expression
  operator: _ @operator)

"->" @operator
".." @operator
"=" @operator

; ── Punctuation ──────────────────────────────────────────────

"(" @punctuation.bracket
")" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket

"," @punctuation.delimiter
":" @punctuation.delimiter
"." @punctuation.delimiter

; ── Attributes ───────────────────────────────────────────────

(attribute
  "@" @attribute
  name: (identifier) @attribute)

; ── Declaration heads ────────────────────────────────────────

(circuit_decl
  name: (identifier) @type.definition)

(module_decl
  name: (identifier) @type.definition)

(model_decl
  name: (identifier) @type.definition)

(model_decl
  device_type: (identifier) @type)

(module_decl
  base: (identifier) @type)

; ── Port declarations ────────────────────────────────────────

(port_decl
  name: (identifier) @variable)

; ── Parameter and let declarations ───────────────────────────

(param_decl
  name: (identifier) @variable)

(param_decl
  type: (identifier) @type)

(let_decl
  name: (identifier) @variable)

(global_decl
  name: (identifier) @variable)

; ── Element and module instances ─────────────────────────────

(element_inst
  name: (identifier) @variable)

(element_inst
  type: (identifier) @type)

(module_inst
  name: (identifier) @variable)

(module_inst
  module: (qualified_name) @type)

; ── Analysis ─────────────────────────────────────────────────

(analysis_decl
  kind: (identifier) @type)

(analysis_setting
  name: (identifier) @property)

(sweep_spec
  source: (identifier) @variable)

; ── Model parameters ─────────────────────────────────────────

(model_param
  name: (identifier) @property)

; ── Named arguments ──────────────────────────────────────────

(named_argument
  name: (identifier) @property)

(named_connection
  name: (identifier) @property)

; ── Block literal entries ────────────────────────────────────

(block_entry
  key: (identifier) @property)

; ── Function calls ───────────────────────────────────────────

(call_expression
  function: (identifier) @function.builtin)

; ── Identifiers (fallback) ───────────────────────────────────

(identifier) @variable
