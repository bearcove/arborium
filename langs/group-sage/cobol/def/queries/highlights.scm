; COBOL highlights query

; Comments
(comment) @comment

; Literals
(string) @string
(h_string) @string
(n_string) @string
(x_string) @string
(number) @number
(integer) @number
(level_number) @number

; Type specifications
(picture_clause) @type
(picture_x) @type.builtin
(picture_9) @type.builtin

; Data definition keywords
[
  (PIC)
  (PICTURE)
  (OCCURS)
] @keyword.storage

; Program structure
(program_name) @function.definition

; Paragraph/section headers (like function definitions)
(paragraph_header
  name: (WORD) @function.definition)
(section_header
  name: (WORD) @function.definition)

; Labels in PERFORM calls (like function calls)
(label
  (qualified_word
    (WORD) @function.call))

; Division/section structure keywords
[
  (IDENTIFICATION)
  (DATA)
  (DIVISION)
  (ENVIRONMENT)
  (CONFIGURATION)
  (WORKING_STORAGE)
  (LINKAGE)
  (LOCAL_STORAGE)
  (INPUT_OUTPUT)
  (FILE_CONTROL)
  (FD)
  (SD)
  (SECTION)
  (PARAGRAPH)
  (PROCEDURE)
  (PROGRAM)
] @keyword.function

; Control flow keywords
[
  (PERFORM)
  (IF)
  (ELSE)
  (THEN)
  (EVALUATE)
  (WHEN)
  (WHEN_OTHER)
  (GO)
  (GOBACK)
  (EXIT)
  (STOP)
  (RUN)
  (CONTINUE)
] @keyword.control

; Data movement keywords
[
  (MOVE)
  (SET)
  (INITIALIZE)
] @keyword

; Arithmetic keywords
[
  (ADD)
  (SUBTRACT)
  (MULTIPLY)
  (DIVIDE)
  (COMPUTE)
] @keyword

; I/O keywords
[
  (ACCEPT)
  (DISPLAY)
  (READ)
  (WRITE)
  (REWRITE)
  (OPEN)
  (CLOSE)
  (DELETE)
  (START)
  (RETURN)
] @keyword

; String operation keywords
[
  (STRING)
  (INSPECT)
] @keyword

; Call/search keywords
[
  (CALL)
  (SEARCH)
  (CANCEL)
  (MERGE)
] @keyword

; Logical operators
[
  (AND)
  (OR)
  (NOT)
] @keyword.operator

; Built-in constants
[
  (TRUE)
  (FALSE)
  (ZERO)
  (ZEROS)
  (SPACE)
  (HIGH_VALUE)
  (LOW_VALUE)
  (TOK_NULL)
  (QUOTE)
  (ALL)
] @constant.builtin

; Type/storage keywords
[
  (BINARY)
  (BINARY_CHAR)
  (BINARY_DOUBLE)
  (BINARY_LONG)
  (BINARY_SHORT)
  (BINARY_C_LONG)
  (COMP)
  (COMP_1)
  (COMP_2)
  (COMP_3)
  (COMP_4)
  (COMP_5)
  (COMP_X)
  (COMPUTATIONAL)
  (PACKED_DECIMAL)
  (POINTER)
  (PROGRAM_POINTER)
  (INDEX)
  (SIGNED)
  (SIGNED_INT)
  (SIGNED_LONG)
  (SIGNED_SHORT)
  (UNSIGNED)
  (UNSIGNED_INT)
  (UNSIGNED_LONG)
  (UNSIGNED_SHORT)
] @type.builtin

; Data type classes
[
  (ALPHABETIC)
  (ALPHABETIC_LOWER)
  (ALPHABETIC_UPPER)
  (ALPHANUMERIC)
  (ALPHANUMERIC_EDITED)
  (NUMERIC)
  (NUMERIC_EDITED)
  (NATIONAL)
  (NATIONAL_EDITED)
] @type

; End keywords
[
  (END_IF)
  (END_PERFORM)
  (END_EVALUATE)
  (END_READ)
  (END_WRITE)
  (END_CALL)
  (END_COMPUTE)
  (END_STRING)
  (END_UNSTRING)
  (END_SEARCH)
  (END_START)
  (END_RETURN)
  (END_REWRITE)
  (END_DELETE)
  (END_ACCEPT)
  (END_DISPLAY)
  (END_ADD)
  (END_SUBTRACT)
  (END_MULTIPLY)
  (END_DIVIDE)
] @keyword

; File/record keywords
[
  (FILE)
  (FILE_ID)
  (RECORD)
  (FILLER)
  (SELECT)
  (ASSIGN)
] @keyword

; Screen/display attributes
[
  (BACKGROUND_COLOR)
  (FOREGROUND_COLOR)
  (HIGHLIGHT)
  (LOWLIGHT)
  (BLINK)
  (REVERSE_VIDEO)
  (UNDERLINE)
  (OVERLINE)
  (BELL)
  (BLANK_LINE)
  (BLANK_SCREEN)
  (ERASE)
  (EOL)
  (EOS)
  (SCROLL)
  (CRT)
  (AUTO)
  (FULL)
  (PROMPT)
  (REQUIRED)
] @attribute

; Positional/directional keywords
[
  (BEFORE)
  (AFTER)
  (FIRST)
  (NEXT)
  (PREVIOUS)
  (UP)
  (DOWN)
  (LEFT)
  (RIGHT)
  (LEADING)
  (TRAILING)
  (ASCENDING)
  (DESCENDING)
  (CORRESPONDING)
] @keyword

; Scope/visibility keywords
[
  (GLOBAL)
  (EXTERNAL)
  (REFERENCE)
] @keyword

; Loop/iteration keywords
[
  (VARYING)
  (THRU)
  (FOREVER)
  (CYCLE)
] @keyword.repeat

; Organization/access keywords
[
  (SEQUENTIAL)
  (RANDOM)
  (DYNAMIC)
  (RELATIVE)
  (INDEXED)
] @keyword

; I/O modes
[
  (INPUT)
  (OUTPUT)
  (I_O)
  (EXTEND)
] @keyword

; Lock/sharing keywords
[
  (LOCK)
  (EXCLUSIVE)
  (MANUAL)
  (AUTOMATIC)
] @keyword

; General keywords
[
  (BY)
  (TO)
  (USING)
  (UPON)
  (ALSO)
  (ANY)
  (ONLY)
  (OPTIONAL)
  (OMITTED)
  (VALUE)
  (SIZE)
  (LENGTH)
  (CHARACTERS)
  (WORDS)
  (LINES)
  (LINE)
  (PAGE)
  (COLUMNS)
  (POSITION)
  (KEY)
  (SORT)
  (SORT_MERGE)
  (DUPLICATES)
  (DEFAULT)
  (ON)
  (OFF)
  (NO)
  (WAIT)
  (ROUNDED)
  (DELIMITED)
  (SEPARATE)
  (SUPPRESS)
  (ADVANCING)
  (DISK)
  (PRINTER)
  (STANDARD)
  (STANDARD_1)
  (STANDARD_2)
  (NATIVE)
  (EBCDIC)
  (REWIND)
  (REEL)
  (UNIT)
  (REMOVAL)
  (PROCEED)
  (NEGATIVE)
  (POSITIVE)
  (INITIALIZED)
  (SOURCE)
  (DATE)
  (DAY)
  (DAY_OF_WEEK)
  (TIME)
  (ENVIRONMENT)
  (ENVIRONMENT_VALUE)
  (ARGUMENT_NUMBER)
  (ARGUMENT_VALUE)
  (COMMAND_LINE)
  (CLASS_NAME)
  (MNEMONIC_NAME)
  (ESCAPE)
  (UPDATE)
  (IGNORE)
  (CHAINING)
] @keyword

; Built-in functions
[
  (CONCATENATE_FUNC)
  (CURRENT_DATE_FUNC)
  (LOCALE_DT_FUNC)
  (LOWER_CASE_FUNC)
  (UPPER_CASE_FUNC)
  (NUMVALC_FUNC)
  (REVERSE_FUNC)
  (SUBSTITUTE_FUNC)
  (SUBSTITUTE_CASE_FUNC)
  (TRIM_FUNCTION)
  (WHEN_COMPILED_FUNC)
] @function.builtin

; Date format specifiers
[
  (YYYYDDD)
  (YYYYMMDD)
] @constant

; Punctuation
(period) @punctuation.delimiter
