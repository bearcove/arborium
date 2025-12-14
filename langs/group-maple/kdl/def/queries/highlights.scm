; --------------------------------------------------------------------------------
; Node + structure
; --------------------------------------------------------------------------------

(node_no_terminator
  (slashdash)? @comment
  name: (string) @type.definition)

((node_no_terminator (slashdash)) @comment
 (#set! "priority" 105))

(node_children) @scope

; Allow top-level identifiers (nodes without string quoting) to appear as types
(identifier_string) @variable

; --------------------------------------------------------------------------------
; Properties / arguments
; --------------------------------------------------------------------------------

(prop key: (string) @property)
(node_prop_or_arg (value (type) @type))

; --------------------------------------------------------------------------------
; Type annotations
; --------------------------------------------------------------------------------

(type) @type
(annotation_type) @type.builtin

; --------------------------------------------------------------------------------
; Literals
; --------------------------------------------------------------------------------

(quoted_string) @string
(raw_string) @string
(escape) @string.escape
(ws_escape) @string.escape

[(boolean) (keyword)] @constant.builtin
(keyword_number) @number.special
(number) @number

; Hash keywords (#true/#false/#null/#inf/etc)
(keyword) @keyword

; --------------------------------------------------------------------------------
; Punctuation / operators
; --------------------------------------------------------------------------------

"=" @operator
["{" "}"] @punctuation.bracket
["(" ")"] @punctuation.bracket
";" @punctuation.delimiter

; --------------------------------------------------------------------------------
; Comments & directives
; --------------------------------------------------------------------------------

[
  (single_line_comment)
  (multi_line_comment)
] @comment @spell

; slashdash directives behave like comments in KDL, so capture the node.
(slashdash) @comment

; Version marker (/- kdl-version ...)
(version_marker) @keyword.directive
