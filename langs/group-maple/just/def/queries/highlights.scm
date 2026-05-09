; This file specifies how matched syntax patterns should be highlighted

[
  "export"
  "import"
] @keyword.control.import

"mod" @keyword.module

[
  "alias"
  "set"
  "shell"
] @keyword

[
  "if"
  "else"
] @keyword.control.conditional

; Variables

(value
  (identifier) @variable)

(alias
  left: (identifier) @variable)

(assignment
  left: (identifier) @variable)

; Functions

(recipe_header
  name: (identifier) @function)

(dependency
  name: (identifier) @function.call)

(dependency_expression
  name: (identifier) @function.call)

(function_call
  name: (identifier) @function.call)

; Parameters

(parameter
  name: (identifier) @variable.parameter)

; Namespaces

(module
  name: (identifier) @namespace)

; Operators

[
  ":="
  "?"
  "=="
  "!="
  "=~"
  "@"
  "="
  "$"
  "*"
  "+"
  "&&"
  "@-"
  "-@"
  "-"
  "/"
  ":"
] @operator

; Punctuation

"," @punctuation.delimiter

[
  "{"
  "}"
  "["
  "]"
  "("
  ")"
  "{{"
  "}}"
] @punctuation.bracket

[ "`" "```" ] @punctuation.special

; Literals

(boolean) @constant.builtin.boolean

[
  (string)
  (external_command)
] @string

(escape_sequence) @constant.character.escape

; Comments

(comment) @spell @comment.line

(shebang) @keyword.directive

; highlight known settings (filtering does not always work)
(setting
  left: (identifier) @keyword
  (#any-of? @keyword
    "allow-duplicate-recipes"
    "allow-duplicate-variables"
    "dotenv-filename"
    "dotenv-load"
    "dotenv-path"
    "dotenv-required"
    "export"
    "fallback"
    "ignore-comments"
    "positional-arguments"
    "shell"
    "shell-interpreter"
    "tempdir"
    "windows-powershell"
    "windows-shell"
    "working-directory"))

; highlight known attributes (filtering does not always work)
(attribute
  (identifier) @attribute
  (#any-of? @attribute
    "confirm"
    "doc"
    "extension"
    "group"
    "linux"
    "macos"
    "metadata"
    "no-cd"
    "no-exit-message"
    "no-quiet"
    "openbsd"
    "parallel"
    "positional-arguments"
    "private"
    "script"
    "unix"
    "windows"
    "working-directory"))

; Numbers are part of the syntax tree, even if disallowed
(numeric_error) @error
