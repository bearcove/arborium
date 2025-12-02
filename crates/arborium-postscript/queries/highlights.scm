; PostScript highlighting queries

; Comments
(comment) @comment
(document_structure_comment) @comment

; Strings
(literal_string) @string
(hexadecimal_string) @string
(base85_string) @string
(escape_sequence) @string.escape

; Numbers
(numeric) @number

; Operators/keywords
(operator) @keyword

; Literals (names)
(literal) @variable

; Punctuation
(array) @punctuation.bracket
(procedure) @punctuation.bracket
(dictionary) @punctuation.bracket
