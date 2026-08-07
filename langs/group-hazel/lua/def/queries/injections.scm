; Parse EmmyLua/LuaCATS documentation comments with the EmmyLuaDoc grammar.
(((comment) @_emmyluadoc_comment
  (#match? @_emmyluadoc_comment "^---")) @injection.content
  (#set! injection.language "emmyluadoc"))

((function_call
  name: [
    (identifier) @_cdef_identifier
    (_ _ (identifier) @_cdef_identifier)
  ]
  arguments: (arguments (string content: _ @injection.content
    (#set! injection.language "c"))))
  (#eq? @_cdef_identifier "cdef"))
