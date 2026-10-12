; Adapted from tong/tree-sitter-haxe queries/highlights.scm. Captures target
; names and tokens rather than whole declarations or #if blocks.

(comment) @comment

((block_comment) @comment.documentation
  (#match? @comment.documentation "^/\\*\\*"))

(identifier) @variable
(type_name) @type

((type_name) @type.builtin
  (#any-of? @type.builtin
    "Any" "Array" "Bool" "Dynamic" "Float" "Int" "Map" "Null" "String" "UInt" "Void"))

(wildcard_pattern) @constant.builtin

["{" "}" "[" "]" "(" ")"] @punctuation.bracket
["<" ">"] @punctuation.bracket
["," ";" ":" "..."] @punctuation.delimiter
["?" "??"] @punctuation.special
["=" "->" "=>"] @operator

; Keywords --------------------------------------------------------------------

[
  "extends"
  "from"
  "implements"
  "package"
  "to"
  "untyped"
  "var"
] @keyword

[
  "abstract"
  "class"
  "enum"
  "interface"
  "typedef"
] @keyword.type

[
  "case"
  "default"
  "else"
  "if"
  "switch"
] @keyword.conditional

[
  "break"
  "continue"
  "do"
  "for"
  "while"
] @keyword.repeat

[
  "catch"
  "throw"
  "try"
] @keyword.exception

[
  "as"
  "cast"
  "in"
  "is"
  "new"
] @keyword.operator

"function" @keyword.function
"return" @keyword.return
["import" "using"] @keyword.import

[
  "dynamic"
  "extern"
  "final"
  "inline"
  "macro"
  "override"
  "private"
  "public"
  "static"
] @keyword.modifier

; Literals --------------------------------------------------------------------

[(Int) (Float)] @number
(String) @string
(String (escape_sequence) @string.escape)
(Regexp) @string.regexp

[(true) (false)] @boolean
(null) @constant.builtin
[(this) (super)] @variable.builtin

(interpolation ["$" "{" "}"] @punctuation.special)

; Modules ---------------------------------------------------------------------

(package_name) @namespace

(import
  path: (package_name) @namespace
  module: (type_name) @type)
(import alias: (identifier) @type)
(import sub: (identifier) @property)
(import (wildcard) @constant)

(using
  path: (package_name) @namespace
  type: (type_name) @type)

; Types -----------------------------------------------------------------------

(ComplexType "->" @operator)
(ComplexType "&" @operator)

(TypePath
  pack: (package_name) @namespace
  name: (type_name) @type
  sub: (identifier) @property)

(FunctionArg name: (identifier) @variable.parameter)

; Declarations ----------------------------------------------------------------

(ClassType name: (type_name) @type.definition)
(AbstractType name: (type_name) @type.definition)
(EnumType name: (type_name) @type.definition)
(DefType name: (type_name) @type.definition)

(ClassType "abstract" @keyword.modifier)

(property_access) @keyword

(ClassMethod name: (identifier) @function.method)
(ClassMethod
  name: (identifier) @constructor
  (#eq? @constructor "new"))
(ClassMethod "macro" name: (identifier) @function.macro)

(EnumConstructor name: (identifier) @constant)

(Field name: (identifier) @variable.member)

; Expressions -----------------------------------------------------------------

(EBinop op: _ @operator)
(EUnop op: _ @operator)
(ETernary ["?" ":"] @operator)

(EField name: (identifier) @property)
(EField "." @punctuation.delimiter)
(ECast type: (ComplexType) @type)
(ECall callee: (identifier) @function.call)
(ECall callee: (EField name: (identifier) @function.call))
(ECall
  callee: (identifier) @function.builtin
  (#eq? @function.builtin "trace"))
(EFunction name: (identifier) @function)
(ENew (TypePath) @type)
(EObjectDecl name: [(identifier) (String)] @property)
(EUntyped "untyped" @keyword.debug)
(EVars "final" @keyword)

; Macros ----------------------------------------------------------------------

"macro" @macro
(reification) @macro
(ClassMethod "macro" @keyword.modifier)
(type_trace "$type" @keyword.debug)

; Metadata --------------------------------------------------------------------

(MetaDataEntry
  [
    "@" @attribute
    ":" @attribute
    name: (identifier) @attribute
  ])

; Conditional compilation -----------------------------------------------------

[
  "#if"
  "#elseif"
  "#else"
  "#error"
] @keyword.directive
(conditional_end) @keyword.directive

(compile_condition (identifier) @constant)
