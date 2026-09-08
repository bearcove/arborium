; Highlights query for EmmyLuaDoc

; Comment Prefix
(comment_prefix) @comment

; Annotation Keywords
"@class" @keyword
"@interface" @keyword
"@field" @keyword
"@type" @keyword
"@param" @keyword
"@return" @keyword
"@generic" @keyword
"@overload" @keyword
"@see" @keyword
"@alias" @keyword
"@enum" @keyword
"@module" @keyword
"@cast" @keyword
"@version" @keyword
"@diagnostic" @keyword
"@operator" @keyword
"@namespace" @keyword
"@using" @keyword
"@language" @keyword
"@attribute" @keyword
"@as" @keyword

; Other/unknown annotations
(tag_name) @keyword

; Special annotation nodes
(deprecated_annotation) @keyword
(private_annotation) @keyword
(protected_annotation) @keyword
(public_annotation) @keyword
(package_annotation) @keyword
(async_annotation) @keyword
(nodiscard_annotation) @keyword
(meta_annotation) @keyword
(readonly_annotation) @keyword
(export_annotation) @keyword

; Visibility modifiers
[
  "public"
  "private"
  "protected"
  "package"
] @keyword.modifier

; Type keywords
[
  "fun"
  "async"
  "table"
  "keyof"
  "typeof"
  "extends"
  "in"
  "and"
  "or"
] @keyword.type

; Identifiers
(identifier) @variable

; Types
(basic_type
  (identifier) @type)

; Built-in types
((identifier) @type.builtin
  (#match? @type.builtin "^(string|number|integer|boolean|table|function|thread|userdata|nil|any|unknown|self)$"))

; Class definitions
(class_annotation
  name: (identifier) @type.definition)

; Class parent types
(class_annotation
  parent: (type_list
    (type
      (primary_type
        (basic_type
          (identifier) @type)))))

; Class modifiers
(class_annotation
  [
    "exact"
    "partial"
    "constructor"
  ] @keyword.modifier)

; Fields and Parameters
(field_annotation
  name: (field_name) @variable.member)

(param_annotation
  name: (param_name) @variable.parameter)

(param_def
  name: (identifier) @variable.parameter)

; Generics
(generic_annotation
  name: (identifier) @type.parameter)

(generic_type
  base: (identifier) @type)

(generic_params
  params: (identifier) @type.parameter)

; Aliases and Enums
(alias_annotation
  name: (identifier) @type.definition)

(enum_annotation
  name: (identifier) @type.definition)

; Enum modifiers
(enum_annotation
  "key" @keyword.modifier)

; Operators
[
  "call"
  "add" "sub" "mul" "div" "mod" "pow"
  "concat"
  "len"
  "eq" "lt" "le"
  "unm"
  "bnot" "band" "bor" "bxor" "shl" "shr"
  "index"
] @operator

; Literals
(string) @string
(number) @number
(boolean) @boolean
"nil" @constant.builtin

; Template types
(template_chars) @string

; Text line (documentation text)
(text_line) @comment

; Description
(description) @comment

; Other annotation description (highlight differently for visibility)
(other_annotation
  description: (description) @string.documentation)

; Continuation description
(continuation_description) @comment

; Punctuation
[
  ":"
  "|"
  ","
  "?"
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "<"
  ">"
  "{"
  "}"
] @punctuation.bracket

; Function type
(function_type
  "fun" @keyword.function)

; Table type
(table_type
  "table" @type.builtin)

; Table fields
(table_field
  name: (identifier) @property)

; Diagnostic actions
(diagnostic_annotation
  action: [
    "disable"
    "enable"
    "disable-next-line"
    "disable-line"
  ] @keyword.directive)

(diagnostic_list
  (identifier) @constant)

; Attributes
(attribute_use_item
  name: (identifier) @function.macro)

(attribute_annotation
  name: (identifier) @function.macro)

; Module names
(module_annotation
  name: (string) @module)

; Version
(version_annotation
  version: [
    (identifier) @constant
    (string) @string
    (version_range) @constant
  ])

; See references
(see_annotation
  reference: (identifier) @variable)

; Namespace
(namespace_annotation
  name: (identifier) @namespace)

; Using
(using_annotation
  path: [
    (identifier) @namespace
    (string) @string
  ])

; Language
(language_annotation
  language: (identifier) @constant)
