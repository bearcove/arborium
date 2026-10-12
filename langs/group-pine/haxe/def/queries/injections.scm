; Adapted from tong/tree-sitter-haxe queries/injections.scm.

((Regexp) @injection.content
  (#set! injection.language "regex"))

((block_comment) @injection.content
  (#set! injection.language "jsdoc"))
