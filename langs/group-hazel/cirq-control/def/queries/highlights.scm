; ngspice .control tree-sitter highlights
; =========================================

; ── Keywords ────────────────────────────────────────────────

[
  "let"
  "echo"
  "if"
  "else"
  "end"
  "foreach"
  "while"
  "repeat"
  "save"
  "quit"
  "set"
  "setplot"
  "define"
  "compose"
  "alter"
  "strcmp"
  "print"
  "write"
  "eprint"
  "eprvcd"
  "stop"
  "when"
  "resume"
  "source"
  "measure"
  "meas"
  "values"
] @keyword

(run_analysis
  kind: _ @keyword)

; ── Comments ────────────────────────────────────────────────

(line_comment) @comment
(inline_comment) @comment

; ── Literals ────────────────────────────────────────────────

(number_literal) @number
(string_literal) @string

; ── Print mode hints ────────────────────────────────────────

(print_mode) @keyword

; ── Variable references ─────────────────────────────────────

(var_ref
  "$" @punctuation.special
  name: (var_name) @variable)

(vec_scalar_ref
  "$&" @punctuation.special
  name: (var_name) @variable)

; ── Vector / device references ──────────────────────────────

(vector_ref
  kind: _ @function.builtin)

(vector_ref
  node: (identifier) @variable)

(vector_ref
  node2: (identifier) @variable)

(device_param) @attribute

; ── Declarations ────────────────────────────────────────────

(let_stmt
  name: (identifier) @variable)

(let_stmt
  name: (indexed_target
    name: (identifier) @variable))

(setplot_stmt
  plot: (identifier) @namespace)

(define_stmt
  name: (identifier) @function)

(define_stmt
  args: (arg_list
    (identifier) @variable.parameter))

(compose_stmt
  name: (identifier) @variable)

(strcmp_stmt
  result: (identifier) @variable)

(measure_stmt
  name: (identifier) @variable)

(measure_stmt
  kind: (identifier) @type)

(stop_when_stmt
  var: (identifier) @variable)

(foreach_stmt
  var: (identifier) @variable)

; ── Operators / punctuation ─────────────────────────────────

(binary_expression
  operator: _ @operator)

(unary_expression
  operator: _ @operator)

"=" @operator
">" @operator
"<" @operator
">=" @operator
"<=" @operator
"<>" @operator

"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket

"," @punctuation.delimiter
":" @punctuation.delimiter

; ── Function calls (fallback after the vector_ref special-case) ──

(call_expression
  function: (identifier) @function.builtin)

; ── Identifiers (fallback) ──────────────────────────────────

(identifier) @variable
