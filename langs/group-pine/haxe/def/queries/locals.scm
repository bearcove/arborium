; Adapted from tong/tree-sitter-haxe queries/locals.scm.

[
  (EBlock)
  (EFunction)
] @local.scope

(FunctionArg
  name: (identifier) @local.definition)

(identifier) @local.reference
