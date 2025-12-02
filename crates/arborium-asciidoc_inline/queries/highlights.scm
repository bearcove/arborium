[
  (monospace)
  (passthrough)
] @markup.raw @nospell

(emphasis) @markup.strong

(ltalic) @markup.italic

(highlight) @markup.italic

[
  (link_url)
  (email)
] @markup.link.url @markup.link

(uri_label) @markup.link.label

[
  "["
  "]"
  "{"
  "}"
  "<<" @punctuation.bracket
  ">>" @punctuation.bracket
] @punctuation.bracket

":" @punctuation.delimiter

(replacement) @string.special

(xref
  (reftext) @markup.link @markup.link.url)

(xref
  (id) @markup.link @markup.link.url .)

(xref
  (id) @markup.link.label
  (reftext) @markup.link @markup.link.url)

[
  (macro_name)
  "((("
  ")))"
  "(("
  "))"
] @keyword

(inline_macro
  (target) @label)

(inline_macro
  (attr) @attribute)

(escaped_sequence) @string.escape

(inline_macro
  (target)? @markup.link @markup.link.url
  (attr)? @label)

(stem_macro
  (target)? @label
  (attr)? @markup.raw @nospell)

(footnote
  (target)? @label
  (attr) @attribute)

(term) @attribute

(id_assignment) @label

(super_escape) @string.special
