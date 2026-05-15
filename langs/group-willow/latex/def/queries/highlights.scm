;; Comments
[
  (comment)
  (line_comment)
  (block_comment)
  (comment_environment)
] @comment

;; Commands — the \commandname token
(command_name) @function

;; Environments — \begin{name} and \end{name}
(begin
  command: _ @function.builtin
  name: (curly_group_text (text) @function.macro))

(end
  command: _ @function.builtin
  name: (curly_group_text (text) @function.macro))

;; Math
(displayed_equation) @markup.raw
(inline_formula) @markup.raw

(math_environment
  (begin
    command: _ @function.builtin
    name: (curly_group_text (text) @markup.raw)))

(math_environment
  (text) @markup.raw)

(math_environment
  (end
    command: _ @function.builtin
    name: (curly_group_text (text) @markup.raw)))

;; Sectioning commands — command gets @namespace, heading text gets @markup.heading
(title_declaration
  command: _ @namespace
  options: (brack_group (_) @markup.heading)?
  text: (curly_group (_) @markup.heading))

(author_declaration
  command: _ @namespace
  authors: (curly_group_author_list
             ((author)+ @markup.heading)))

(chapter
  command: _ @namespace
  toc: (brack_group (_) @markup.heading)?
  text: (curly_group (_) @markup.heading))

(part
  command: _ @namespace
  toc: (brack_group (_) @markup.heading)?
  text: (curly_group (_) @markup.heading))

(section
  command: _ @namespace
  toc: (brack_group (_) @markup.heading)?
  text: (curly_group (_) @markup.heading))

(subsection
  command: _ @namespace
  toc: (brack_group (_) @markup.heading)?
  text: (curly_group (_) @markup.heading))

(subsubsection
  command: _ @namespace
  toc: (brack_group (_) @markup.heading)?
  text: (curly_group (_) @markup.heading))

(paragraph
  command: _ @namespace
  toc: (brack_group (_) @markup.heading)?
  text: (curly_group (_) @markup.heading))

(subparagraph
  command: _ @namespace
  toc: (brack_group (_) @markup.heading)?
  text: (curly_group (_) @markup.heading))

;; Definitions and references
(new_command_definition
  command: _ @function.macro
  declaration: (curly_group_command_name (command_name) @function))
(new_command_definition
  command: _ @function.macro
  declaration: (command_name) @function)
(old_command_definition
  command: _ @function.macro
  declaration: (_) @function)
(let_command_definition
  command: _ @function.macro
  declaration: (_) @function)

(environment_definition
  command: _ @function.macro
  name: (curly_group_text (text) @constant))

(theorem_definition
  command: _ @function.macro
  name: (curly_group_text_list (text) @constant))

(paired_delimiter_definition
  command: _ @function.macro
  declaration: (curly_group_command_name (command_name) @function))

;; Labels and references
(label_definition
  command: _ @function.macro
  name: (curly_group_label (label) @label))
(label_reference_range
  command: _ @function.macro
  from: (curly_group_label (label) @label)
  to: (curly_group_label (label) @label))
(label_reference
  command: _ @function.macro
  names: (curly_group_label_list (label) @label))
(label_number
  command: _ @function.macro
  name: (curly_group_label (label) @label)
  number: (curly_group) @markup.link)

;; Citations
(citation
  command: _ @function.macro
  keys: (curly_group_text_list) @string)

;; Glossary and acronyms
(glossary_entry_definition
  command: _ @function.macro
  name: (curly_group_text (text) @string))
(glossary_entry_reference
  command: _ @function.macro
  name: (curly_group_text (text) @string))
(acronym_definition
  command: _ @function.macro
  name: (curly_group_text (text) @string))
(acronym_reference
  command: _ @function.macro
  name: (curly_group_text (text) @string))

;; Colors
(color_definition
  command: _ @function.macro
  name: (curly_group_text (text) @string))
(color_reference
  command: _ @function.macro
  name: (curly_group_text (text) @string))

;; File inclusion
(class_include
  command: _ @keyword.storage.type
  path: (curly_group_path) @string)

(package_include
  command: _ @keyword.storage.type
  paths: (curly_group_path_list) @string)

(latex_include
  command: _ @keyword.control.import
  path: (curly_group_path) @string)
(import_include
  command: _ @keyword.control.import
  directory: (curly_group_path) @string
  file: (curly_group_path) @string)

(bibtex_include
  command: _ @keyword.control.import
  paths: (curly_group_path_list) @string)
(biblatex_include
  "\\addbibresource" @keyword.control.import
  glob: (curly_group_glob_pattern) @string)

(graphics_include
  command: _ @keyword.control.import
  path: (curly_group_path) @string)
(tikz_library_import
  command: _ @keyword.control.import
  paths: (curly_group_path_list) @string)

;; Hyperlinks (\url{...} and \href{...}{...})
(hyperlink
  command: _ @function.macro
  uri: (curly_group_uri) @markup.link)

;; Text formatting
((generic_command
  command: (command_name) @_name
  arg: (curly_group (_) @markup.italic))
  (#eq? @_name "\\emph"))

((generic_command
  command: (command_name) @_name
  arg: (curly_group (_) @markup.italic))
  (#match? @_name "^(\\\\textit|\\\\mathit)$"))

((generic_command
  command: (command_name) @_name
  arg: (curly_group (_) @markup.bold))
  (#match? @_name "^(\\\\textbf|\\\\mathbf)$"))

((generic_command
  command: (command_name) @_name
  .
  arg: (curly_group (_) @markup.link))
  (#match? @_name "^(\\\\url|\\\\href)$"))

;; Key-value parameters
(key_value_pair
  key: (_) @variable.parameter)

[
  (brack_group)
  (brack_group_argc)
] @variable.parameter

;; Operators and punctuation
[(operator) "="] @operator

"\\item" @punctuation.special

(delimiter) @punctuation.delimiter

["[" "]" "{" "}"] @punctuation.bracket

(math_delimiter
  left_command: _ @punctuation.delimiter
  left_delimiter: _ @punctuation.delimiter
  right_command: _ @punctuation.delimiter
  right_delimiter: _ @punctuation.delimiter)
