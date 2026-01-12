function sepBy(pattern, separator) {
  return seq(pattern, repeat(seq(separator, pattern)))
}

function nonempty(pattern1, pattern2) {
  return choice(
    seq(pattern1, optional(pattern2)),
    pattern2
  )
}

module.exports = grammar({
  name: 'cobol',
  word: $ => $._WORD,
  externals: $ => [
    $._WHITE_SPACES,
    $._LINE_PREFIX_COMMENT,
    $._LINE_SUFFIX_COMMENT,
    $._LINE_COMMENT,
    $.comment_entry,
    $._multiline_string,
    $.exec_statement,
  ],

  extras: $ => [
    $._WHITE_SPACES,
    $._LINE_PREFIX_COMMENT,
    $._LINE_SUFFIX_COMMENT,
    $._LINE_COMMENT,
    $.comment,
  ],

  rules: {
    start: $ => choice(
      repeat1($.program_definition),
      $.copybook_definition
    ),

    // Standalone copybook files (no IDENTIFICATION DIVISION)
    copybook_definition: $ => repeat1(
      choice(
        seq($.data_description, repeat1('.')),
        $.copy_statement,
        $.exec_statement
      )
    ),

    _LINE_COMMENT_ALIAS: $ => alias($._LINE_COMMENT, $.comment),

    comment: $ => /\*>[^\n]*/,

    program_definition: $ => prec.right(seq(
      $.identification_division,
      optional($.environment_division),
      optional($.data_division),
      optional($.procedure_division), //todo
      repeat($.end_program), //todo
      //optional($.LINE_PREFIX_COMMENT),
    )),

    identification_division: $ => seq(
      $.IDENTIFICATION, $.DIVISION, '.',
      optional(
        seq($._PROGRAM_ID, '.',
          $.program_name,
          optional(choice(
            $.as_literal,
            $.is_initial,
            $.is_common)),
          '.')),
      repeat(choice(
        $.author_section,
        $.installation_section,
        $.date_written_section,
        $.date_compiled_section,
        $.security_section)),
    ),

    program_name: $ => choice(
      $._WORD,
      $._LITERAL
    ),

    as_literal: $ => seq(
      $._AS,
      $._LITERAL
    ),

    is_initial: $ => seq(
      $._IS,
      $._INITIAL
    ),

    is_common: $ => seq(
      $._IS,
      $._COMMON
    ),

    author_section: $ => seq(
      $._AUTHOR, '.',
      field('comment', repeat1($.comment_entry)),
    ),

    installation_section: $ => seq(
      $._INSTALLATION, '.',
      field('comment', repeat1($.comment_entry)),
    ),

    date_written_section: $ => seq(
      $._DATE_WRITTEN, '.',
      field('comment', repeat1($.comment_entry)),
    ),

    date_compiled_section: $ => seq(
      $._DATE_COMPILED, '.',
      field('comment', repeat1($.comment_entry)),
    ),

    security_section: $ => seq(
      $._SECURITY, '.',
      field('comment', repeat1($.comment_entry)),
    ),

    function_definition: $ => seq(
      $.function_division,
      $.environment_division,
      $.data_division,
      $.procedure_division,
      $.end_function
    ),

    function_division: $ => seq(
      $._FUNCTION_ID,
      '.',
      $.program_name,
      $.as_literal,
      '.'
    ),

    end_function: $ => seq(
      $._END_FUNCTION,
      $.program_name,
      '.'
    ),

    environment_division: $ => seq(
      $.ENVIRONMENT,
      $.DIVISION,
      '.',
      optional(choice(
        seq(optional($.configuration_section), optional($.input_output_section)),
        $.copy_statement  // COPY can provide entire environment division content
      )),
    ),

    configuration_section: $ => seq(
      $.CONFIGURATION,
      $.SECTION,
      '.',
      repeat($._configuration_paragraph)
    ),

    _configuration_paragraph: $ => choice(
      $.source_computer_paragraph,
      $.object_computer_paragraph,
      $.special_names_paragraph,
      $.repository_paragraph,
    ),

    source_computer_paragraph: $ => seq(
      $._SOURCE_COMPUTER,
      '.',
      optional($._source_computer_entry),
    ),

    _source_computer_entry: $ => choice(
      seq($.WORD, '.'),
      seq($.WORD, $.with_debugging_mode, '.'),
      seq($.with_debugging_mode, '.')
    ),

    with_debugging_mode: $ => seq(
      optional($._WITH),
      $._DEBUGGING,
      $._MODE
    ),

    object_computer_paragraph: $ => seq(
      $._OBJECT_COMPUTER,
      '.',
      optional($._object_computer_entry)
    ),

    _object_computer_entry: $ => choice(
      seq($.WORD, '.'),
      seq($.WORD, repeat1($._object_clause), '.'),
      seq(repeat1($._object_clause), '.')
    ),

    _object_clause: $ => choice(
      $.object_computer_memory,
      $.object_computer_sequence,
      $.object_computer_segment
    ),

    object_computer_memory: $ => seq(
      $._MEMORY,
      $._SIZE,
      optional($._IS),
      $.integer,
      choice(
        $.CHARACTERS,
        $.WORDS
      )
    ),

    object_computer_sequence: $ => seq(
      optional($._PROGRAM),
      $._coll_sequence,
      optional($._IS),
      $.qualified_word
    ),

    _coll_sequence: $ => seq(
      optional($._COLLATING),
      $._SEQUENCE
    ),

    object_computer_segment: $ => seq(
      $._SEGMENT_LIMIT,
      optional($._IS),
      $.integer
    ),

    special_names_paragraph: $ => seq(
      $._SPECIAL_NAMES,
      '.',
      repeat(seq($.special_name, optional('.'))),
    ),

    special_name: $ => choice(
      $.mnemonic_name_clause,
      $.alphabet_name_clause,
      $.symbolic_characters_clause,
      $.locale_clause,
      $.class_name_clause,
      $.currency_sign_clause,
      $.decimal_point_clause,
      $.cursor_clause,
      $.crt_status_clause,
      $.screen_control,
      $.event_status,
    ),

    mnemonic_name_clause: $ => choice(
      seq(
        field('crt', $.WORD),
        optional($._IS),
        $._CRT
      ),
      seq(
        field('word', $.WORD),
        optional($._IS),
        field('value', $.WORD),
        repeat($.special_name_mnemonic_on_off),
      ),
      seq(
        field('word', $.WORD),
        optional($._IS),
        repeat1($.special_name_mnemonic_on_off),
      ),
      seq(
        $._ARGUMENT_NUMBER,
        optional($._IS),
        field('argument_number', $.WORD),
      ),
      seq(
        $._ARGUMENT_VALUE,
        optional($._IS),
        field('argument_value', $.WORD),
      ),
      seq(
        $._ENVIRONMENT_NAME,
        optional($._IS),
        field('environment_name', $.WORD),
      ),
      seq(
        $._ENVIRONMENT_VALUE,
        optional($._IS),
        field('environment_value', $.WORD),
      ),
    ),

    special_name_mnemonic_on_off: $ => seq(
      choice($.ON, $.OFF),
      optional($._STATUS),
      optional($._IS),
      $.WORD
    ),

    alphabet_name_clause: $ => seq(
      $._ALPHABET,
      field('word', $.WORD),
      optional($._IS),
      field('value', $._alphabet_definition),
    ),

    _alphabet_definition: $ => choice(
      $.NATIVE,
      $.STANDARD_1,
      $.STANDARD_2,
      $.EBCDIC,
      repeat1($.alphabet_literal),
    ),

    alphabet_literal: $ => choice(
      sepBy($._alphabet_lits, $.ALSO),
      seq($._alphabet_lits, $.THRU, $._alphabet_lits),
    ),

    _alphabet_lits: $ => choice(
      $.LITERAL,
      $.SPACE,
      $.ZERO,
      $.QUOTE,
      $.HIGH_VALUE,
      $.LOW_VALUE
    ),

    symbolic_characters_clause: $ => seq(
      $._SYMBOLIC,
      optional($._CHARACTERS),
      field('char_list', repeat1($.WORD)),
      optional(choice($._IS, $._ARE)),
      field('integer_list', repeat1($.integer)),
    ),

    locale_clause: $ => seq(
      $._LOCALE,
      $.WORD,
      optional($._IS),
      $.qualified_word,
    ),

    class_name_clause: $ => seq(
      $._CLASS,
      field('x', $.WORD),
      optional($._IS),
      field('list', repeat1($.class_item))
    ),

    class_item: $ => seq(
      $._basic_value,
      optional(seq($._THRU, $._basic_value))
    ),

    currency_sign_clause: $ => seq(
      $._CURRENCY,
      optional($._SIGN),
      optional($._IS),
      $._LITERAL
    ),

    decimal_point_clause: $ => seq(
      $._DECIMAL_POINT,
      optional($._IS),
      $._COMMA,
    ),

    cursor_clause: $ => seq(
      $._CURSOR,
      optional($._IS),
      $.qualified_word
    ),

    crt_status_clause: $ => seq(
      $._CRT,
      $._STATUS,
      optional($._IS),
      $.qualified_word
    ),

    screen_control: $ => seq(
      $._SCREEN,
      $._CONTROL,
      optional($._IS),
      $.qualified_word
    ),

    event_status: $ => seq(
      $._EVENT_STATUS,
      optional($._IS),
      $.qualified_word
    ),

    repository_paragraph: $ => seq(
      $._REPOSITORY,
      '.',
      optional($._repository)
    ),

    _repository: $ => seq(
      repeat1($.repository_name),
      '.'
    ),

    repository_name: $ => seq(
      $._FUNCTION,
      choice(
        repeat1($.LITERAL),
        $.ALL
      ),
      $._INTRINSIC
    ),

    input_output_section: $ => choice(
      seq(
        $.INPUT_OUTPUT, $.SECTION, '.',
        optional($._file_control_paragraph),
        optional($._i_o_control_paragraph)
      ),
      $._file_control_paragraph,
      $._i_o_control_paragraph,
    ),

    _file_control_paragraph: $ => seq(
      $.FILE_CONTROL, '.',
      repeat($.select_statement)
    ),

    select_statement: $ => seq(
      $.SELECT,
      field('optional', optional($.OPTIONAL)),
      field('file_name', $.WORD),
      repeat($._select_clause),
      '.'
    ),

    _i_o_control_paragraph: $ => seq(
      $._I_O_CONTROL, '.', optional($.i_o_control),
    ),

    i_o_control: $ => choice(
      seq(repeat1($.i_o_control_clause), optional('.')),
      seq(repeat1($.apply_clause), optional('.')),
    ),

    i_o_control_clause: $ => choice(
      $.same_clause,
      $.multiple_file_tape_clause,
      $.i_o_control_header,
    ),

    // todo this i_o_control does not exits
    i_o_control_header: $ => seq(
      $.WORD, '.'
    ),

    same_clause: $ => prec.right(seq(
      $._SAME,
      field('option', choice(
        $.RECORD,
        $.SORT,
        $.SORT_MERGE
      )),
      optional($._AREA),
      optional($._FOR),
      field('file_name_list', repeat1($.WORD))
    )),

    multiple_file_tape_clause: $ => prec.right(seq(
      $._MULTIPLE,
      optional($._FILE),
      optional($._TAPE),
      optional($._CONTAINS),
      repeat1($.multiple_file)
    )),

    multiple_file: $ => seq(
      field('file_name', $.WORD),
      optional(seq($.POSITION, $.integer))
    ),

    apply_clause: $ => seq(
      $._APPLY,
      choice(
        seq(
          $._COMMITMENT_CONTROL,
          optional($._ON),
          field('commitment_control', repeat1($.qualified_word)),
        ),
        seq(
          $._CYL_OVERFLOW,
          optional($._OF),
          field('cyl_overflow', $._LITERAL),
          $._TRACKS,
          $._ON,
          field('tracks_on', repeat1($.qualified_word)),
        ),
        seq(
          $._CORE_INDEX,
          $._TO,
          field('core_index_to', $.qualified_word),
          $._ON,
          field('core_index_on', repeat1($.qualified_word))
        ),
        seq(
          $._FORMS_OVERLAY,
          $._TO,
          field('forms_overlay_to', $.qualified_word),
          $._ON,
          field('forms_overlay_on', repeat1($.qualified_word))
        ),
        seq(
          $._CLOSE_NOFEED,
          $._ON,
          field('close_nofeed_on', repeat1($.qualified_word)),
        ),
      )
    ),

    _select_clause: $ => choice(
      $.assign_clause,
      $.access_mode_clause,
      $.alternative_record_key,
      $.collating_sequence_clause,
      $.file_status_clause,
      $.lock_mode_clause,
      $.organization_clause,
      $.padding_character_clause,
      $.record_delimiter_clause,
      $.record_key_clause,
      $.relative_key_clause,
      $.reserve_clause,
      $.sharing_clause,
      $.error,
      $.nominal_key_clause,
    ),

    assign_clause: $ => seq(
      $.ASSIGN,
      optional($._TO),
      field('external_or_dynamic', optional($._ext_clause)),
      field('to', choice(
        $.DISK,
        $.PRINTER,
        seq(optional($._device), $._assignment_name),
      ))
    ),

    _ext_clause: $ => choice(
      $.EXTERNAL,
      $.DYNAMIC
    ),

    _device: $ => choice(
      $.DISK,
      $.PRINTER,
    ),

    _assignment_name: $ => choice(
      $._LITERAL,
      $.DISPLAY,
      seq(
        optional($._LITERAL), repeat1($.qualified_word)
      )
    ),

    access_mode_clause: $ => seq(
      $._ACCESS,
      optional($._MODE),
      optional($._IS),
      choice(
        $.SEQUENTIAL,
        $.DYNAMIC,
        $.RANDOM
      )
    ),

    alternative_record_key: $ => seq(
      $._ALTERNATE,
      $._RECORD,
      optional($._KEY),
      optional($._IS),
      field('reference', $.qualified_word),
      field('with_dups', optional($.with_dups)),
    ),

    with_dups: $ => seq(
      optional($._WITH),
      $._DUPLICATES
    ),

    collating_sequence_clause: $ => seq(
      $._coll_sequence,
      optional($._IS),
      $.WORD
    ),

    _coll_sequence: $ => seq(
      optional($._COLLATING),
      $._SEQUENCE
    ),

    file_status_clause: $ => seq(
      field('file_or_sort', optional($._file_or_sort)),
      $._STATUS,
      optional($._IS),
      field('reference', $.qualified_word),
      field('reference2', optional($.qualified_word)),
    ),

    _file_or_sort: $ => choice(
      $.FILE,
      $.SORT
    ),

    lock_mode_clause: $ => seq(
      $._LOCK,
      optional($._MODE),
      optional($._IS),
      $.lock_mode
    ),

    lock_mode: $ => choice(
      seq(
        choice($.MANUAL, $.AUTOMATIC),
        field('lock_with', optional($._lock_with))
      ),
      $.EXCLUSIVE
    ),

    _lock_with: $ => (
      seq(
        $._WITH, $._LOCK, $._ON,
        optional($.MULTIPLE),
        choice($._RECORD, $._RECORDS)
      ),
      seq($._WITH, $.RECORD)
    ),

    organization_clause: $ => seq(
      optional(seq($._ORGANIZATION, optional($._IS))),
      choice(
        $.INDEXED,
        seq($.RECORD, optional($._BINARY), $.SEQUENTIAL),
        $.SEQUENTIAL,
        $.RELATIVE,
      )
    ),

    padding_character_clause: $ => seq(
      $._PADDING,
      optional($._CHARACTER),
      optional($._IS),
      choice($.qualified_word, $._LITERAL)
    ),

    record_delimiter_clause: $ => seq(
      $._RECORD,
      $._DELIMITER,
      optional($._IS),
      $._STANDARD_1
    ),

    record_key_clause: $ => seq(
      $._RECORD,
      optional($._KEY),
      optional($._IS),
      field('reference', $.qualified_word),
      optional(seq(
        field('key_is_eq', optional(choice(
          seq($.SOURCE, $._IS),
          '='
        ))
        ),
        field('split_key_list', repeat1($.qualified_word))
      ))
    ),

    relative_key_clause: $ => seq(
      $._RELATIVE,
      optional($._KEY),
      optional($._IS),
      $.qualified_word
    ),

    reserve_clause: $ => seq(
      $._RESERVE,
      choice(
        seq($.integer, optional(choice($._AREA, $._AREAS))),
        $.NO
      )
    ),

    sharing_clause: $ => seq(
      $._SHARING,
      optional($._WITH),
      choice(
        seq($.ALL, optional($._OTHER)),
        seq($.NO, optional($._OTHER)),
        seq($.READ, $.ONLY),
      )
    ),

    error: $ => /todo_error/,

    nominal_key_clause: $ => seq(
      $._NOMINAL,
      optional($._KEY),
      optional($._IS),
      $.qualified_word
    ),

    data_division: $ => seq(
      $.DATA, $.DIVISION, '.',
      optional($.file_section),
      optional($.working_storage_section),
      optional($.local_storage_section),
      optional($.linkage_section),
      optional($.report_section),
      optional($.screen_section),
    ),

    file_section: $ => choice(
      seq(
        $._FILE,
        $._SECTION,
        '.',
        repeat($.file_description)
      ),
      seq(
        $.file_type,
        seq(
          $.file_description_entry,
          optional($.record_description_list),
          repeat1($.file_description)
        )
      )
    ),

    file_description: $ => choice(
      seq(
        $.file_type,
        $.file_description_entry,
        $.record_description_list
      ),
      // FD name COPY book. - COPY may be followed by additional record descriptions
      seq(
        $.file_type,
        $.WORD,
        $.copy_statement,
        optional($.record_description_list)
      )
    ),

    file_type: $ => choice(
      $.FD,
      $.SD
    ),

    file_description_entry: $ => seq(
      $.WORD,
      repeat($.file_description_clause),
      '.'
    ),

    file_description_clause: $ => choice(
      seq(optional($._IS), $.EXTERNAL),
      seq(optional($._IS), $.GLOBAL),
      $.block_contains_clause,
      $.record_clause,
      $.label_records_clause,
      $.value_of_clause,
      $.data_records_clause,
      $.linage_clause,
      $.recording_mode_clause,
      $.code_set_clause,
      $.report_clase,
      $.error,
    ),

    block_contains_clause: $ => prec.left(seq(
      $._BLOCK,
      optional($._CONTAINS),
      field('num', $.integer),
      field('to', optional(seq($._TO, $.integer))),
      optional(choice($._CHARACTERS, $._RECORDS))
    )),

    record_clause: $ => choice(
      seq(
        $._RECORD,
        optional($._CONTAINS),
        field('num', $.integer),
        field('to', optional(seq($._TO, $.integer))),
        optional($._CHARACTERS)
      ),
      seq(
        $._RECORD,
        optional($._IS),
        $.VARYING,
        optional($._IN),
        optional($._SIZE),
        field('from', optional(seq(optional($._FROM), $.integer))),
        field('to', optional(seq($._TO, $.integer))),
        optional($._CHARACTERS),
        field('depend', optional(seq($._DEPENDING, optional($._ON), $.qualified_word))),
      )
    ),

    label_records_clause: $ => seq(
      $._LABEL,
      $._records,
      choice($.STANDARD, $.OMITTED)
    ),

    _records: $ => prec.left(choice(
      seq($._RECORD, optional($._IS)),
      seq($._RECORDS, optional($._ARE)),
    )),

    value_of_clause: $ => seq(
      $._VALUE, $._OF,
      field('name', choice($.WORD, $.FILE_ID)),
      optional($._IS),
      field('value', choice($._LITERAL, $.qualified_word))
    ),

    data_records_clause: $ => seq(
      $._DATA,
      $._records,
      repeat1($.qualified_word),
    ),

    linage_clause: $ => seq(
      $._LINAGE,
      optional($._IS),
      field('reference', choice($.qualified_word, $._LITERAL)),
      optional($._LINES),
      field('lines', repeat($.linage_lines)),
    ),

    linage_lines: $ => choice(
      $.linage_footing,
      $.linage_top,
      $.linage_bottom
    ),

    linage_footing: $ => seq(
      optional($._WITH),
      $._FOOTING,
      optional($._AT),
      field('reference', choice($.qualified_word, $._LITERAL)),
      optional($._LINES)
    ),

    linage_top: $ => seq(
      optional($._AT),
      $._TOP,
      field('reference', choice($.qualified_word, $._LITERAL)),
      optional($._LINES)
    ),

    linage_bottom: $ => seq(
      optional($._AT),
      $._BOTTOM,
      field('reference', choice($.qualified_word, $._LITERAL)),
    ),

    recording_mode_clause: $ => seq(
      $._RECORDING,
      optional($._MODE),
      optional($._IS),
      field('mode', $.WORD)
    ),

    code_set_clause: $ => seq(
      $._CODE_SET,
      optional($._IS),
      field('code_set', $.WORD)
    ),

    report_clase: $ => choice(
      seq($._REPORT, optional($._IS), field('name', $.WORD)),
      seq($._REPORTS, optional($._ARE), field('name', $.WORD)),
    ),

    record_description_list: $ => repeat1(
      choice(
        seq($.data_description, repeat1('.')),
        // level [name] COPY book. [clauses]. - COPY may be followed by additional clauses
        seq($.level_number, optional($.entry_name), $._copy_clause,
            repeat($._data_description_clause), '.'),
        $.copy_statement,
        $.exec_statement
      )
    ),

    working_storage_section: $ => seq(
      $.WORKING_STORAGE, $.SECTION, '.',
      repeat(choice(
        seq($.data_description, repeat1('.')),
        seq($.level_number, optional($.entry_name), $._copy_clause,
            repeat($._data_description_clause), '.'),
        $.copy_statement,
        $.exec_statement
      ))
    ),

    data_description: $ => choice(
      $.constant_entry,
      seq(
        $.level_number, $.entry_name,
        repeat($._data_description_clause), optional($._LITERAL)
      ),
      seq(
        $.level_number,
        repeat1($._data_description_clause), optional($._LITERAL)
      ),
      /*seq(
        $.level_number_88, $.entry_name
      ),*/
    ),

    level_number: $ => /[0-9][0-9]?/,
    level_number_88: $ => /88/,

    entry_name: $ => choice(
      $._FILLER,
      $._WORD
    ),

    constant_entry: $ => seq(
      $.level_number,
      field('name', $.WORD),
      $._CONSTANT,
      field('is_global', seq(optional($._IS), $.GLOBAL)),
      optional($._AS),
      $._lit_or_length,
    ),

    _lit_or_length: $ => choice(
      $._literal,
      field('length_of', seq($._LEADING, optional($._OF), $._identifier)),
      field('byte_length_of', seq($._BYTE_LENGTH, optional($._OF), $._identifier)),
    ),

    _data_description_clause: $ => choice(
      $.redefines_clause,
      $.external_clause,
      $.global_clause,
      $.picture_clause,
      $.usage_clause,
      $.sign_clause,
      $.occurs_clause,
      $.justified_clause,
      $.synchronized_clause,
      $.blank_clause,
      $.based_clause,
      $.value_clause,
      $.renames_clause,
      $.any_length_clause,
      $.error
    ),

    redefines_clause: $ => seq(
      $._REDEFINES, $._identifier
    ),

    _identifier: $ => prec.right(choice(
      $.qualified_word,
      seq($.qualified_word, $.subref),
      seq($.qualified_word, $.refmod),
      seq($.qualified_word, $.subref, $.refmod),
    )),

    qualified_word: $ => sepBy(
      $.WORD, $._in_of
    ),

    _in_of: $ => choice(
      $._IN,
      $._OF
    ),

    subref: $ => seq(
      '(',
      $._exp_list,
      ')'
    ),

    refmod: $ => seq(
      '(',
      $._exp,
      ':',
      optional($._exp),
      ')'
    ),

    _exp_list: $ => repeat1($._exp),

    _exp: $ => choice(
      $._arith_x,
      $._binary_exp,
      $._unary_exp,
      seq('(', $._exp, ')')
    ),

    _binary_exp: $ => choice(
      $.pow_exp,
      $.mul_exp,
      $.div_exp,
      $.add_exp,
      $.sub_exp,
    ),

    _unary_exp: $ => prec(4, choice(
      $.positive_exp,
      $.negative_exp
    )),

    positive_exp: $ => seq(
      '+',
      field('value', $._exp)
    ),

    negative_exp: $ => seq(
      '-',
      field('value', $._exp)
    ),

    add_exp: $ => prec.left(1, seq(
      field('left', $._exp),
      '+',
      field('right', $._exp)
    )),

    sub_exp: $ => prec.left(1, seq(
      field('left', $._exp),
      '-',
      field('right', $._exp)
    )),

    mul_exp: $ => prec.left(2, seq(
      field('left', $._exp),
      '*',
      field('right', $._exp)
    )),

    div_exp: $ => prec.left(2, seq(
      field('left', $._exp),
      '/',
      field('right', $._exp)
    )),

    pow_exp: $ => prec.left(3, seq(
      field('left', $._exp),
      '^',
      field('right', $._exp)
    )),

    //todo
    _arith_x: $ => choice(
      $._identifier,
      seq($.LENGTH, optional($._OF), $._identifier),
      seq($.LENGTH, optional($._OF), $._basic_literal),
      seq($.LENGTH, optional($._OF), $.function_),
      $._basic_literal,
      $.function_,
      $.linage_counter,
    ),

    external_clause: $ => seq(
      optional($._IS),
      $.EXTERNAL,
      optional(seq(
        $._AS,
        $._LITERAL,
      ))
    ),

    global_clause: $ => seq(
      optional($._IS),
      $.GLOBAL
    ),

    picture_clause: $ => seq(
      choice($.PICTURE, $.PIC),
      optional($._IS),
      $._picture_string
    ),

    //todo
    _picture_string: $ => choice(
      $.picture_x,
      $.picture_n,
      $.picture_9,
      $.picture_a,
      $.picture_edit,
    ),

    picture_x: $ => /([xX](\([0-9]+\))?)+/,

    picture_n: $ => /([nN](\([0-9]+\))?)+/,

    picture_9: $ => choice(
      $._picture_9_z,
      $._picture_9_v_1,
      $._picture_9_v_2,
    ),
    _picture_9_z: $ => /[sS]?(9(\([0-9]+\))?)+([zZ](\([0-9]+\))?)+/,
    _picture_9_v_1: $ => /[sS]?([pP9](\([0-9]+\))?)+([vV]([pP9](\([0-9]+\))?)*)?/,
    _picture_9_v_2: $ => /[sS]?[vV]([pP9](\([0-9]+\))?)*/,

    picture_a: $ => /([aA](\([0-9]+\))?)+/,

    picture_edit: $ => /([aAxX9bBvVzZpPwW\(\)0-9$/,\.*+<>-]|[cC][rR]|[dD][bB])*([aAxX9bBvVzZpPwW\(\)0-9$/,*+<>-]|[cC][rR]|[dD][bB])/,

    usage_clause: $ => seq(
      optional(seq($._USAGE, optional($._IS))),
      $._usage
    ),

    _usage: $ => choice(
      $.BINARY,
      $.COMPUTATIONAL,
      $.COMP,
      $.COMP_1,
      $.COMP_2,
      $.COMP_3,
      $.COMP_4,
      $.COMP_5,
      $.COMP_X,
      $.DISPLAY,
      $.INDEX,
      $.PACKED_DECIMAL,
      $.POINTER,
      $.PROGRAM_POINTER,
      $.SIGNED_SHORT,
      $.SIGNED_INT,
      $.SIGNED_LONG,
      $.UNSIGNED_SHORT,
      $.UNSIGNED_INT,
      $.UNSIGNED_LONG,
      seq($.BINARY_CHAR, $.SIGNED),
      seq($.BINARY_CHAR, $.UNSIGNED),
      $.BINARY_CHAR,
      seq($.BINARY_SHORT, $.SIGNED),
      seq($.BINARY_SHORT, $.UNSIGNED),
      $.BINARY_SHORT,
      seq($.BINARY_LONG, $.SIGNED),
      seq($.BINARY_LONG, $.UNSIGNED),
      $.BINARY_LONG,
      seq($.BINARY_DOUBLE, $.SIGNED),
      seq($.BINARY_DOUBLE, $.UNSIGNED),
      $.BINARY_DOUBLE,
      seq($.BINARY_C_LONG, $.SIGNED),
      seq($.BINARY_C_LONG, $.UNSIGNED),
      $.BINARY_C_LONG,
      $.NATIONAL
    ),

    sign_clause: $ => seq(
      optional($._SIGN),
      optional($._IS),
      choice($.LEADING, $.TRAILING),
      optional(seq($.SEPARATE, optional($._CHARACTER)))
    ),

    occurs_clause: $ => seq(
      $.OCCURS,
      field('num', $.integer),
      field('to', optional(seq($._TO, $.integer))),
      optional($._TIMES),
      field('depending', optional(seq($._DEPENDING, optional($._ON), $.qualified_word))),
      field('key_spec', optional($.occurs_key_spec)),
    ),

    occurs_key_spec: $ => choice(
      seq(repeat1($.occurs_key), optional($.occurs_indexed)),
      seq($.occurs_indexed, repeat($.occurs_key))
    ),

    occurs_key: $ => seq(
      choice($.ASCENDING, $.DESCENDING),
      optional($._KEY),
      optional($._IS),
      field('keys', repeat1($.qualified_word))
    ),

    occurs_indexed: $ => seq(
      $._INDEXED,
      optional($._BY),
      repeat1($.WORD)
    ),

    justified_clause: $ => seq(
      choice($._JUSTIFIED, $._JUST),
      optional($.RIGHT)
    ),

    synchronized_clause: $ => seq(
      choice($._SYNCHRONIZED, $._SYNC),
      optional(choice($.LEFT, $.RIGHT))
    ),

    blank_clause: $ => seq(
      $._BLANK,
      optional($._WHEN),
      $._ZERO,
    ),

    based_clause: $ => $._BASED,

    value_clause: $ => prec.right(seq(
      choice($.VALUE, $._VALUES),
      optional(choice($._IS, $._ARE)),
      repeat1($.value_item),
      optional($._WHEN),
      optional($._SET),
      optional($._TO),
      optional(seq(
        $._FALSE,
        optional($._IS),
        field('when_set_to_false', $._literal)))
    )),

    value_item: $ => seq(
      $._literal,
      optional(seq(
        $.THRU,
        $._literal
      ))
    ),

    value_when_clause: $ => seq(
    ),

    renames_clause: $ => seq(
      $._RENAMES,
      $.qualified_word,
      optional(seq($._THRU, $.qualified_word))
    ),

    any_length_clause: $ => seq(
      $._ANY,
      $._LENGTH
    ),

    local_storage_section: $ => seq(
      $.LOCAL_STORAGE,
      $.SECTION,
      '.',
      $.record_description_list
    ),

    linkage_section: $ => seq(
      $.LINKAGE, $.SECTION, '.',
      $.record_description_list
    ),

    report_section: $ => /report_section/,
    screen_section: $ => /screen_section/,

    //todo
    procedure_division: $ => seq(
      $.PROCEDURE, $.DIVISION,
      optional($.procedure_using_chaining),
      optional($.procedure_returning),
      '.',
      optional($.procedure_declaratives),
      optional($._procedure_division_contenet)
    ),

    procedure_using_chaining: $ => seq(
      choice($.USING, $.CHAINING),
      repeat($.procedure_param),
    ),

    procedure_param: $ => seq(
      optional($.procedure_type),
      optional($.size),
      optional($.OPTIONAL),
      $._WORD
    ),

    procedure_type: $ => seq(
      optional($._BY),
      choice($.VALUE, $.REFERENCE)
    ),

    size: $ => choice(
      seq($.SIZE, optional($._IS), $.AUTO),
      seq($.SIZE, optional($._IS), $.DEFAULT),
      seq($.UNSIGNED, $.SIZE, optional($._IS), $.integer),
      seq($.SIZE, optional($._IS), $.integer),
    ),

    procedure_type: $ => seq(
      optional($._BY),
      choice($.REFERENCE, $.VALUE)
    ),

    procedure_returning: $ => seq(
      $._RETURNING,
      $._WORD
    ),

    procedure_declaratives: $ => seq(
      $._DECLARATIVES, '.',
      optional($._procedure_division_declaratives_content),
      $._END, $._DECLARATIVES, '.'
    ),

    _procedure_division_headers: $ => repeat1(choice(
      $.section_header,
      $.paragraph_header,
    )),

    _procedure_division_statements_before_header: $ => prec(1, seq(
      repeat($._procedure_division_statement),
      $._end_statement
    )),

    _procedure_division_contenet: $ => prec.right(choice(
      seq(
        optional($._procedure_division_headers),
        repeat(seq(
          $._procedure_division_statements_before_header,
          $._procedure_division_headers,
        )),
        repeat1($._procedure_division_statement)
      ),
      seq(
        optional($._procedure_division_headers),
        repeat1(seq(
          $._procedure_division_statements_before_header,
          $._procedure_division_headers,
        )),
        repeat($._procedure_division_statement)
      ),
    )),

    _procedure_division_declaratives_content: $ => prec.right(1, choice(
      $._procedure_division_headers,
      seq(
        optional($._procedure_division_headers),
        repeat1(seq(
          $._procedure_division_statements_before_header,
          $._procedure_division_headers,
        )),
      ),
      seq(
        optional($._procedure_division_headers),
        repeat(seq(
          $._procedure_division_statements_before_header,
          $._procedure_division_headers,
        )),
        $._procedure_division_statements_before_header,
      ),
    )),

    _procedure_division_statement: $ => choice(
      $._statement,
      $._start_handler,
      $._end_statement,
    ),

    _procedure: $ => choice(
      $.section_header,
      $.paragraph_header,
      $._statement,
      $._start_handler,
      $._end_statement,
    ),

    section_header: $ => seq(
      field('name', choice($.WORD, $.integer)),
      $._SECTION,
      optional($._LITERAL),
      '.'
    ),

    paragraph_header: $ => seq(
      field('name', choice($.WORD, $.integer)),
      '.'
    ),

    end_program: $ => prec(1, seq(
      $._END_PROGRAM,
      $.program_name,
      '.'
    )),

    _statement: $ => choice(
      $.accept_statement,
      $.add_statement,
      $.allocate_statement,
      $.alter_statement,
      $.call_statement,
      $.cancel_statement,
      $.close_statement,
      $.continue_statement,
      $.compute_statement,
      $.copy_statement,
      $.delete_statement,
      $.display_statement,
      $.divide_statement,
      $.exec_statement,
      $.exit_statement,
      $.goback_statement,
      $.goto_statement,
      $.initialize_statement,
      $.inspect_statement,
      $.merge_statement,
      $.move_statement,
      $.multiply_statement,
      $.open_statement,
      $.perform_statement_call_proc,
      $.read_statement,
      $.release_statement,
      $.return_statement,
      $.rewrite_statement,
      $.search_statement,
      $.set_statement,
      $.sort_statement,
      $.start_statement,
      $.stop_statement,
      $.string_statement,
      $.subtract_statement,
      $.unstring_statement,
      $.use_statement,
      $.write_statement,
      $.next_sentence_statement,
    ),

    _end_statement: $ => choice(
      $.END_ACCEPT,
      $.END_ADD,
      $.END_CALL,
      $.END_COMPUTE,
      $.END_DELETE,
      $.END_DISPLAY,
      $.END_DIVIDE,
      $.END_MULTIPLY,
      $.END_READ,
      $.END_SEARCH,
      $.END_SUBTRACT,
      $.END_WRITE,
      $.END_PERFORM,
      $.END_IF,
      $.END_EVALUATE,
      $.END_RETURN,
      $.END_REWRITE,
      $.END_START,
      $.END_STRING,
      $.END_UNSTRING,
      $.period
    ),

    period: $ => '.',

    _start_handler: $ => choice(
      $.on_exception,
      $.not_on_exception,
      $.on_size_error,
      $.not_on_size_error,
      $.on_overflow,
      $.not_on_overflow,
      $.at_end,
      $.not_at_end,
      $.eop,
      $.not_eop,
      $.invalid_key,
      $.not_invalid_key,

      $.evaluate_header,
      $.when,
      $.when_other,
      $.perform_statement_loop,
      $.if_header,
      $.else_if_header,
      $.else_header,
    ),

    // COPY with period - used as standalone statement
    copy_statement: $ => seq(
      $._copy_clause,
      '.'
    ),

    // COPY without period - used inline where more content may follow
    _copy_clause: $ => prec.right(seq(
      $._COPY,
      field('book', optional(choice($.WORD, $.string))),
      field('lib_name', optional(seq(
        $._in_of,
        choice($.WORD, $.string)))
      ),
      field('supress', optional($.SUPPRESS)),
      optional($.replacing_clause),
    )),

    replacing_clause: $ => seq(
      field('leading_or_trailing', optional(choice($.LEADING, $.TRAILING))),
      field('x', choice($.WORD, $.string)),
      optional($._BY),
      field('by', choice($.WORD, $.string)),
    ),

    start_statement: $ => seq(
      $.START,
      field('file_name', $.WORD),
      optional($.start_key),
    ),

    start_key: $ => seq(
      $._KEY,
      field('op', choice(
        $.eq,
        $.gt,
        $.lt,
        $.ne,
        $.ge,
        $.ne
      )),
      field('keys', repeat1($._identifier))
    ),

    stop_statement: $ => choice(
      seq(
        $.STOP,
        $.RUN,
        optional(choice(
          field('returning', seq($._RETURNING, $._x)),
          field('giving', seq($._GIVING, $._x))
        ))
      ),
      seq(
        $.STOP,
        field('x', $._simple_value),
      )
    ),

    accept_statement: $ => seq(
      $.ACCEPT,
      $._accept_body,
    ),

    _accept_body: $ => prec.right(seq(
      $._identifier,
      choice(
        seq(
          optional($.at_line_column),
          optional($.with_accp_attr),
        ),
        seq(
          $._FROM,
          field('from', choice(
            seq($.ESCAPE, $.KEY),
            $.LINES,
            $.COLUMNS,
            seq($.DATE, optional($.YYYYMMDD)),
            seq($.DAY, optional($.YYYYDDD)),
            $.DAY_OF_WEEK,
            $.TIME,
            $.COMMAND_LINE,
            $.ENVIRONMENT_VALUE,
            $.ENVIRONMENT,
            $.ARGUMENT_NUMBER,
            $.ARGUMENT_VALUE,
            $.MNEMONIC_NAME,
            $.WORD
          ))
        )
      )
    )),

    at_line_column: $ => choice(
      seq(optional($._AT), $.line_number, $.column_number),
      seq(optional($._AT), $.column_number, $.line_number),
      seq(optional($._AT), $.column_number),
      seq(optional($._AT), $.line_number),
      seq($._AT, $._simple_value)
    ),

    line_number: $ => seq(
      $._LINE,
      optional($._NUMBER),
      $._id_or_lit,
    ),

    column_number: $ => seq(
      choice($._POSITION, $._COLUMN),
      optional($._NUMBER),
      $._id_or_lit,
    ),

    with_accp_attr: $ => seq(
      $._WITH, repeat1(choice(
        $.BELL,
        $.BLINK,
        $.HIGHLIGHT,
        $.LOWLIGHT,
        $.REVERSE_VIDEO,
        $.UNDERLINE,
        $.OVERLINE,
        seq(choice($.BACKGROUND_COLOR, $.FOREGROUND_COLOR), optional($._IS), $._num_or_id_or_lit),
        seq($.SCROLL, choice($.UP, $.DOWN), optional(seq(
          optional($.BY),
          $._num_or_id_or_lit,
          choice($.LINE, $.LINES)))),
        $.AUTO,
        $.FULL,
        $.REQUIRED,
        $.UPDATE,
        $.PROMPT
      ))
    ),

    add_statement: $ => seq(
      $.ADD,
      $._add_body,
    ),

    _add_body: $ => seq(
      choice(
        seq(
          field('from', repeat1($._x)),
          $._TO,
          field('to', repeat1($.arithmetic_x)),
        ),
        seq(
          field('from', repeat1($._x)),
          field('to', optional(seq($._TO, $._x))),
          $._GIVING,
          field('giving', repeat1($.arithmetic_x)),
        ),
        seq(
          $.CORRESPONDING,
          field('from', $._identifier),
          $._TO,
          field('to', seq($._identifier, optional($.ROUNDED))),
        )
      ),
    ),

    _size_error_block: $ => nonempty(
      $.on_size_error,
      $.not_on_size_error,
    ),

    on_size_error: $ => seq(
      $._ON,
      $._SIZE,
      $._ERROR,
    ),

    not_on_size_error: $ => seq(
      $._NOT,
      $._ON,
      $._SIZE,
      $._ERROR,
    ),

    allocate_statement: $ => prec(1, seq(
      $._ALLOCATE,
      choice(
        seq(
          field('x', $.WORD),
          field('initialized', optional($.INITIALIZED)),
          field('returning', optional(seq($._RETURNING, $._target_x)))
        ),
        seq(
          field('x', $.expr),
          $.CHARACTERS,
          field('initialized', optional($.INITIALIZED)),
          field('returning', optional(seq($._RETURNING, $._target_x)))
        )
      )
    )),

    alter_statement: $ => seq(
      $._ALTER,
      repeat1($.alter_option)
    ),

    alter_option: $ => seq(
      field('proc_name', $._id_or_lit),
      $.TO,
      optional(seq(
        $.PROCEED,
        $.TO
      )),
      field('to', $._id_or_lit)
    ),

    call_statement: $ => seq(
      $._call_header
    ),

    _call_header: $ => seq(
      $.CALL,
      field('x', $._id_or_lit_or_func),
      field('using', optional(seq(
        $._USING,
        repeat1($._call_param)
      ))),
      optional(choice(
        field('returning', seq($._RETURNING, $._identifier),
          field('giving', seq($._GIVING, $._identifier)),
        )))
    ),

    _call_param: $ => choice(
      $.call_param_omitted,
      $.call_param_arg,
    ),

    call_param_omitted: $ => seq(
      optional($._call_type), $._OMITTED
    ),

    call_param_arg: $ => seq(
      optional($._call_type),
      optional($._size_option),
      $._x
    ),

    _call_type: $ => choice(
      $.by_reference,
      $.by_content,
      $.by_value,
    ),

    by_reference: $ => seq(optional($._BY), $._REFERENCE),
    by_content: $ => seq(optional($._BY), $._CONTENT),
    by_value: $ => seq(optional($._BY), $._VALUE),

    _size_option: $ => choice(
      $.size_is_auto,
      $.size_is_default,
      $.unsigned_size_is,
      $.size_is,
    ),

    size_is_auto: $ => seq($._SIZE, optional($._IS), $._AUTO),
    size_is_default: $ => seq($._SIZE, optional($._IS), $._DEFAULT),
    unsigned_size_is: $ => seq($._UNSIGNED, $._SIZE, optional($._IS), $.integer),
    size_is: $ => seq($._SIZE, optional($._IS), $.integer),

    on_exception: $ => seq(
      optional($._ON),
      $._EXCEPTION,
    ),

    on_overflow: $ => seq(
      optional($._ON),
      $._OVERFLOW,
    ),

    not_on_exception: $ => seq(
      $._NOT,
      optional($._ON),
      $._EXCEPTION,
    ),

    not_on_overflow: $ => seq(
      $._NOT,
      optional($._ON),
      $._OVERFLOW,
    ),

    cancel_statement: $ => seq(
      $.CANCEL,
      field('all', optional($.ALL)),
      repeat($._id_or_lit)
    ),

    close_statement: $ => seq(
      $.CLOSE,
      repeat1($.close_arg),
    ),

    compute_statement: $ => seq(
      $.COMPUTE,
      field('left', repeat1($.arithmetic_x)),
      choice('=', $._EQUAL),
      field('right', $.expr),
    ),

    close_arg: $ => seq(
      field('file_handler', $.WORD),
      optional($.close_option)
    ),

    close_option: $ => choice(
      seq(
        choice($.REEL, $.UNIT),
        optional($._close_removal)
      ),
      seq(optional($._WITH), $.NO, $.REWIND),
      seq(optional($._WITH), $.LOCK),
    ),

    _close_removal: $ => seq(
      optional($._FOR),
      $.REMOVAL
    ),

    continue_statement: $ => $.CONTINUE,

    delete_statement: $ => seq(
      $.DELETE,
      field('file_name', $.WORD),
      optional(choice($._RECORD, $._RECORDS)),
    ),

    display_statement: $ => seq(
      $.DISPLAY,
      $._display_body,
    ),

    _display_body: $ => prec.right(choice(
      seq($._id_or_lit, $._UPON_ENVIRONMENT_NAME),
      seq($._id_or_lit, $._UPON_ENVIRONMENT_VALUE),
      seq($._id_or_lit, $._UPON_ARGUMENT_NUMBER),
      seq($._id_or_lit, $._UPON_COMMAND_LINE),
      seq(repeat1($._x), optional($.at_line_column), optional($.with_clause)),
      seq(repeat1($._x), optional($.at_line_column), $.UPON, $.MNEMONIC_NAME, optional($.with_clause)),
      seq(repeat1($._x), optional($.at_line_column), $.UPON, $._WORD, optional($.with_clause)),
      seq(repeat1($._x), optional($.at_line_column), $.UPON, $.PRINTER, optional($.with_clause)),
      seq(repeat1($._x), optional($.at_line_column), $.UPON, $.CRT, optional($.with_clause)),
    )),

    at_line_column: $ => choice(
      seq(optional($._AT), $.line_number, $.column_number),
      seq(optional($._AT), $.column_number, $.line_number),
      seq(optional($._AT), $.line_number),
      seq(optional($._AT), $.column_number),
      seq($._AT, field('at', $._simple_value))
    ),

    line_number: $ => seq(
      $._LINE,
      optional($.number),
      $._id_or_lit
    ),

    column_number: $ => choice(
      seq(
        $._COLUMN,
        field('column_x', optional($.number)),
        field('column_y', $._id_or_lit),
      ),
      seq(
        $._POSITION,
        field('position_x', optional($.number)),
        field('position_y', $._id_or_lit),
      ),
    ),

    _id_or_lit: $ => choice(
      $._identifier,
      $._literal
    ),

    with_clause: $ => choice(
      seq(optional($._WITH), $.NO, $.ADVANCING),
      seq($._WITH, repeat1($.disp_attr))
    ),

    disp_attr: $ => choice(
      $.BELL,
      $.BLINK,
      seq($.ERASE, $.EOL),
      seq($.ERASE, $.EOS),
      $.HIGHLIGHT,
      $.LOWLIGHT,
      $.REVERSE_VIDEO,
      $.UNDERLINE,
      $.OVERLINE,
      seq(
        choice($.FOREGROUND_COLOR, $.BACKGROUND_COLOR),
        optional($._IS),
        $._num_or_id_or_lit),
      seq(
        $.SCROLL,
        choice($.UP, $.DOWN),
        optional($._IS), optional($.scroll_lines)),
      $.BLANK_LINE,
      $.BLANK_SCREEN,
    ),

    _num_or_id_or_lit: $ => choice(
      $._identifier,
      $.number,//todo convert this to integer
      $.ZERO
    ),

    scroll_lines: $ => seq(
      optional($._BY),
      $._num_or_id_or_lit,
      optional($.line_or_lines),
    ),

    line_or_lines: $ => choice($.LINE, $.LINES),

    divide_statement: $ => seq(
      $.DIVIDE,
      $._divide_body,
    ),

    _divide_body: $ => seq(
      field('x', $._x),
      choice(
        seq($._INTO, field('into', repeat1($.arithmetic_x))),
        seq($._INTO, field('into', $._x), $._GIVING, field('giving', repeat1($.arithmetic_x))),
        seq($._BY, field('by', $._x), $._GIVING, field('giving', repeat1($.arithmetic_x))),
        seq($._INTO, field('into', $._x), $._GIVING, field('giving', $.arithmetic_x), $._REMAINDER, field('remainder', $.arithmetic_x)),
        seq($._BY, field('by', $._x), $._GIVING, field('giving', $.arithmetic_x), $._REMAINDER, field('remainder', $.arithmetic_x)),
      )
    ),

    evaluate_header: $ => prec.right(seq(
      $.EVALUATE,
      sepBy($.evaluate_subject, optional($._ALSO)),
    )),

    evaluate_subject: $ => choice(
      $.expr,
      $.TRUE,
      $.FALSE,
    ),

    when: $ => prec.right(repeat1(seq($.WHEN, $._evaluate_object_list))),

    _evaluate_object_list: $ => prec.right(sepBy($._evaluate_object, optional($._ALSO))),

    _evaluate_object: $ => choice(
      $.expr,
      $.ANY,
      $.TRUE,
      $.FALSE
    ),

    when_other: $ => $.WHEN_OTHER,

    exit_statement: $ => prec.left(seq(
      $.EXIT,
      optional(choice(
        $.PROGRAM,
        $.PERFORM,
        seq($.PERFORM, $.CYCLE),
        $.SECTION,
        $.PARAGRAPH
      ))
    )),

    goback_statement: $ => $.GOBACK,

    goto_statement: $ => seq(
      $.GO, optional($._TO),
      field('to', repeat($.label)),
      field('depending', optional(seq($._DEPENDING, optional($._ON), $._identifier)))
    ),

    label: $ => choice(
      $.qualified_word,
      seq($._LITERAL, optional(seq($._in_of, $._LITERAL)))
    ),


    if_header: $ => prec(1, seq(
      $.IF,
      field('condition', choice($.expr)),
      optional($.THEN),
    )),

    else_if_header: $ => prec.right(1, seq(
      $.ELSE, $._IF,
      field('condition', choice($.expr)),
      optional($.THEN),
    )),

    else_header: $ => $.ELSE,

    expr: $ => prec.left(choice(
      seq($.NOT, $.expr),
      seq($.expr, choice($.AND, $.OR), $.expr),
      $._expr_bool,
      seq("(", $.expr, ")")
    )),

    _expr_data: $ => $._x,

    _expr_calc: $ => prec(1, choice(
      $._expr_calc_binary,
      $._expr_calc_unary,
      $._expr_data,
      seq("(", $._expr_calc, ")")
    )),

    _expr_calc_binary: $ => choice(
      prec.left(1, seq($._expr_calc, '+', $._expr_calc)),
      prec.left(1, seq($._expr_calc, '-', $._expr_calc)),
      prec.left(2, seq($._expr_calc, '**', $._expr_calc)),
      prec.left(2, seq($._expr_calc, '*', $._expr_calc)),
      prec.left(2, seq($._expr_calc, '/', $._expr_calc)),
      prec.left(3, seq($._expr_calc, '^', $._expr_calc)),
    ),

    _expr_calc_unary: $ => prec(4, choice(
      seq('+', $._expr_calc),
      seq('-', $._expr_calc),
      seq('^', $._expr_calc),
    )),

    _expr_compare: $ => //choice(
      //prec.left(-1, seq($._expr_calc, $.ne, $._expr_calc)),
      //prec.left(-1, seq($._expr_calc, $.eq, $._expr_calc)),
      //prec.left(-1, seq($._expr_calc, $.gt, $._expr_calc)),
      //prec.left(-1, seq($._expr_calc, $.lt, $._expr_calc)),
      //prec.left(-1, seq($._expr_calc, $.ge, $._expr_calc)),
      //prec.left(-1, seq($._expr_calc, $.le, $._expr_calc)),
      //),
      prec.left(-1, seq(
        $._expr_calc,
        $._comparator,
        $._expr_calc,
        repeat(seq(
          choice(
            $.AND_LT,
            $.AND_LE,
            $.AND_GT,
            $.AND_GE,
            $.AND_EQ,
            $.AND_NE,
            $.OR_LT,
            $.OR_LE,
            $.OR_GT,
            $.OR_GE,
            $.OR_EQ,
            $.OR_NE,
          ),
          $._expr_calc
        ))
      )),

    _comparator: $ => choice(
      $.ne, $.eq, $.gt, $.lt, $.ge, $.le
    ),

    _expr_is: $ => prec(1, choice(
      seq($._expr_calc, optional($._IS), choice(
        $.OMITTED,
        $.NUMERIC,
        $.ALPHABETIC,
        $.ALPHABETIC_LOWER,
        $.ALPHABETIC_UPPER,
        $.CLASS_NAME,
        $.POSITIVE,
        $.NEGATIVE,
        $.ZERO,

        $.NOT_OMITTED,
        $.NOT_NUMERIC,
        $.NOT_ALPHABETIC,
        $.NOT_ALPHABETIC_LOWER,
        $.NOT_ALPHABETIC_UPPER,
        $.NOT_CLASS_NAME,
        $.NOT_POSITIVE,
        $.NOT_NEGATIVE,
        $.NOT_ZERO
      ))
    )),

    _expr_bool: $ => choice(
      $._expr_is,
      $._expr_compare,
      $._expr_calc,
      $.is_class,
      $.is_not_class,
    ),

    is_class: $ => prec(1, seq(
      field('x', $._x),
      field('class', $.WORD)
    )),

    is_not_class: $ => prec(1, seq(
      field('x', $._x),
      $._NOT,
      field('class', $.WORD)
    )),

    eq: $ => seq(
      optional($._IS),
      choice($._EQUAL, '='),
      optional($._TO)
    ),

    ge: $ => choice(
      seq(optional($._IS), '>='),
      seq(optional($._IS), $._NOT, '<'),
      seq(optional($._IS), $._GREATER, optional($._THAN), optional($._OR), $._EQUAL, optional($._TO)),
      seq(optional($._IS), $._NOT_LESS, optional($._THAN)),
    ),

    le: $ => choice(
      seq(optional($._IS), '<='),
      seq(optional($._IS), $._NOT, '>'),
      seq(optional($._IS), $._LESS, optional($._THAN), optional($._OR), $._EQUAL, optional($._TO)),
      seq(optional($._IS), $._NOT_GREATER, optional($._THAN)),
    ),

    ne: $ => seq(optional($._IS), $._NOT_EQUAL, optional($._TO)),


    gt: $ => choice(
      seq(optional($._IS), '>'),
      seq(optional($._IS), $._NOT, '<='),
      seq(optional($._IS), $._GREATER, optional($._THAN))
    ),

    lt: $ => choice(
      seq(optional($._IS), '<'),
      seq(optional($._IS), $._NOT, '>='),
      seq(optional($._IS), $._LESS, optional($._THAN))
    ),

    initialize_statement: $ => seq(
      $.INITIALIZE,
      field('x', repeat1($._target_x)),
      field('filler', optional(seq(optional($._WITH), $.FILLER))),
      field('value', optional(choice(
        $.all_to_value,
        $.initialize_category_to_value))),
      field('replacing', optional(seq(
        $._REPLACING,
        repeat1($.initialize_replacing_item)))),
      field('default', optional($.DEFAULT)),
    ),

    all_to_value: $ => seq(
      $._ALL,
      optional($._TO),
      $._VALUE
    ),

    initialize_category_to_value: $ => seq(
      $._initialize_category,
      optional($._TO),
      $._VALUE
    ),

    _initialize_category: $ => choice(
      $.ALPHABETIC,
      $.ALPHANUMERIC,
      $.NUMERIC,
      $.ALPHANUMERIC_EDITED,
      $.NUMERIC_EDITED,
      $.NATIONAL,
      $.NATIONAL_EDITED
    ),

    initialize_replacing_item: $ => seq(
      field('category', $._initialize_category),
      optional($._DATA),
      $._BY,
      field('by', $._x),
    ),

    inspect_statement: $ => seq(
      $.INSPECT,
      field('send', choice(
        $._identifier,
        $._literal,
        $.function_
      )),
      repeat1(choice(
        $.inspect_tallying,
        $.inspect_replacing,
        $.inspect_converting,
      ))
    ),

    inspect_tallying: $ => seq(
      $._TALLYING,
      repeat1(choice(
        field('for', seq($._simple_value, $._FOR)),
        field('character', seq($._CHARACTERS, optional($.inspect_region))),
        field('all', $.ALL),
        field('leading', $.LEADING),
        field('trailing', $.TRAILING),
        field('two', seq($._simple_value, optional($.inspect_region))),
      ))
    ),

    inspect_replacing: $ => seq(
      $._REPLACING,
      repeat1($.replacing_item)
    ),

    replacing_item: $ => choice(
      field('characters_by', seq(
        $._CHARACTERS, $._BY,
        $._simple_value, optional($.inspect_region)
      )),
      field('region', seq(
        optional(choice(
          $.ALL,
          $.LEADING,
          $.FIRST,
          $.TRAILING
        )),
        $.replacing_region
      )),
    ),

    replacing_region: $ => seq(
      field('x', $._simple_value),
      $._BY,
      field('by', $._simple_all_value),
      field('region', optional($.inspect_region))
    ),

    inspect_converting: $ => seq(
      $._CONVERTING,
      field('x', $._simple_value),
      $._TO,
      field('to', $._simple_all_value),
      field('region', optional($.inspect_region))
    ),

    inspect_region: $ => repeat1(seq(
      choice($.BEFORE, $.AFTER),
      optional($._INITIAL),
      field('x', $._x)
    )),

    _simple_all_value: $ => choice(
      $._identifier,
      $._literal
    ),

    merge_statement: $ => seq(
      $.MERGE,
      $._sort_body
    ),

    move_statement: $ => seq(
      $.MOVE,
      $._move_body
    ),

    _move_body: $ => seq(
      optional($._CORRESPONDING),
      field('src', $._x),
      $._TO,
      field('dst', $._target_x_list)
    ),

    _x: $ => choice(
      seq($._LENGTH, optional($._OF), choice(
        $._identifier,
        $._basic_literal,
        $.function_)
      ),
      seq($._ADDRESS, optional($._OF), choice(
        seq(
          choice($._PROGRAM, $._ENTRY),
          choice($._identifier, $._LITERAL)),
        $._identifier,
      )),
      $._literal,
      $.function_,
      $.linage_counter,
      $._identifier,
    ),

    arithmetic_x: $ => seq(
      $._x,
      optional($.ROUNDED)
    ),

    _target_x_list: $ => repeat1($._target_x),
    _target_x: $ => choice(
      $._identifier,
      seq($._ADDRESS, optional($._OF), $._identifier)
    ),

    multiply_statement: $ => seq(
      $.MULTIPLY,
      $._multiply_body,
    ),

    _multiply_body: $ => choice(
      seq(
        field('val1', $._x),
        $._BY,
        field('val2', repeat1($.arithmetic_x)),
      ),
      seq(
        field('val1', $._x),
        $._BY,
        field('val2', $._x),
        $._GIVING,
        field('giving', repeat1($.arithmetic_x)),
      )
    ),

    open_statement: $ => seq(
      $.OPEN,
      repeat1($.open_arg)
    ),

    open_arg: $ => seq(
      field('mode', choice($.INPUT, $.OUTPUT, $.I_O, $.EXTEND)),
      field('sharing', optional(seq(
        $._SHARING,
        optional($._WITH),
        choice(
          seq($.ALL, optional($._OTHER)),
          seq($.NO, optional($._OTHER)),
          seq($.READ, $.ONLY),
        )
      ))),
      field('file_name_list', repeat1($.WORD)),
      field('option', optional(choice(
        seq(optional($._WITH), $.NO, $.REWIND),
        seq(optional($._WITH), $.LOCK)
      )))
    ),

    perform_statement_call_proc: $ => seq(
      $.PERFORM,
      field('procedure', $.perform_procedure),
      field('option', optional($.perform_option)),
    ),

    perform_statement_loop: $ => seq(
      $.PERFORM,
      field('option', optional($.perform_option)),
    ),

    perform_procedure: $ => seq(
      $.label,
      optional(seq($.THRU, $.label)),
    ),

    perform_option: $ => choice(
      $.FOREVER,
      seq(field('times', $._id_or_lit_or_func), $._TIMES),
      seq(field('test', optional($.perform_test)), $._UNTIL, field('until', $.expr)),
      seq(field('test', optional($.perform_test)), $._VARYING, field('varying', sepBy($.perform_varying, $._AFTER)))
    ),

    _id_or_lit_or_func: $ => choice(
      $._identifier,
      $._LITERAL,
      $.function_
    ),

    perform_test: $ => seq(
      optional($._WITH),
      $._TEST,
      choice($.BEFORE, $.AFTER)
    ),

    perform_varying: $ => seq(
      $._identifier,
      $._FROM,
      field('from', $._x),
      $._BY,
      field('by', $._x),
      $._UNTIL,
      field('until', $.expr),
    ),

    _read_statement_header: $ => prec.right(seq(
      $._READ,
      field('file_name', $.WORD),
      field('flag_next', optional(choice($.NEXT, $.PREVIOUS))),
      optional($._RECORD),
      field('into', optional(seq($._INTO, $._identifier))),
      optional($.with_lock),
      field('key', optional(seq(
        $._KEY,
        optional($._IS),
        repeat1($._identifier),
      ))),
    )),

    read_statement: $ => $._read_statement_header,

    with_lock: $ => choice(
      seq($._IGNORING, $._LOCK),
      seq(optional($._WITH), choice(
        $.LOCK,
        seq($.NO, $.LOCK),
        seq($.IGNORE, $.LOCK),
        $.WAIT
      ))
    ),

    at_end: $ => seq(optional($._AT), $._END),
    not_at_end: $ => seq($._NOT, optional($._AT), $._END),

    release_statement: $ => seq(
      $._RELEASE,
      field('record', $.qualified_word),
      field('from', optional(seq($._FROM, $._id_or_lit)))
    ),

    return_statement: $ => seq(
      $.RETURN,
      field('file_name', $.WORD),
      optional($._RECORD),
      field('into', optional(seq($._INTO, $._identifier)))
    ),

    rewrite_statement: $ => seq(
      $.REWRITE,
      field('record', $.qualified_word),
      field('from', optional(seq($._FROM, $._id_or_lit))),
      field('lock', optional(choice($.write_lock, $.write_no_lock))),
    ),

    search_statement: $ => seq(
      $.SEARCH,
      optional(field('all', $.ALL)),
      field('table_name', $.qualified_word),
      field('varying', optional(seq($.VARYING, $._identifier)))
    ),

    set_statement: $ => seq(
      $.SET,
      choice(
        $.set_environment,
        $.set_to,
        $.set_up_down,
        repeat1($.set_to_on_off),
        repeat1($.set_to_true_false),
      )
    ),

    set_environment: $ => seq(
      $._ENVIRONMENT,
      field('environment', $._simple_value),
      optional($._TO),
      field('to', $._simple_value)
    ),

    set_to: $ => choice(
      seq(
        field('from', $._target_x_list),
        $._TO,
        $._ENTRY,
        field('to_entry', $._alnum_or_id)
      ),
      seq(
        field('from', $._target_x_list),
        $._TO,
        field('to', $._x)
      )
    ),

    _alnum_or_id: $ => choice(
      $._identifier,
      $._LITERAL,
    ),

    set_up_down: $ => seq(
      field('x', $._target_x_list),
      choice($.UP, $.DOWN),
      $._BY,
      field('by', $._x)
    ),

    set_to_on_off: $ => seq(
      field('mnemonic_names', repeat1($.MNEMONIC_NAME)),
      $._TO,
      choice($.ON, $.OFF)
    ),

    set_to_true_false: $ => seq(
      field('x', $._target_x_list),
      $._TO,
      choice($._TRUE, $._FALSE)
    ),

    sort_statement: $ => seq(
      $._SORT,
      $._sort_body,
    ),

    _sort_body: $ => prec.right(seq(
      field('x', $.qualified_word),
      field('key_list', repeat($.sort_key)),
      field('duplicates', optional(seq(
        optional($._WITH),
        $.DUPLICATES,
        optional($._IN,),
        $._ORDER
      ))),
      field('collating', optional(seq(
        $._coll_sequence,
        optional($._IS),
        $.qualified_word
      ))),
      field('input', optional(choice(
        $.sort_input_using,
        $.sort_input_procedure
      ))),
      field('output', optional(choice(
        $.sort_output_giving,
        $.sort_output_procedure,
      )))
    )),

    sort_key: $ => seq(
      optional($._ON),
      field('order', choice($.ASCENDING, $.DESCENDING)),
      optional($._KEY),
      optional($._IS),
      field('key_list', repeat($.qualified_word))
    ),

    sort_input_using: $ => seq(
      $._USING,
      repeat1($.WORD)
    ),

    sort_input_procedure: $ => seq(
      $._INPUT,
      $._PROCEDURE,
      optional($._IS),
      $.perform_procedure,
    ),

    sort_output_giving: $ => seq(
      $._GIVING,
      repeat1($.WORD)
    ),

    sort_output_procedure: $ => seq(
      $._OUTPUT,
      $._PROCEDURE,
      optional($._IS),
      $.perform_procedure,
    ),

    string_statement: $ => seq(
      $.STRING,
      field('from', repeat1($.string_item)),
      $._INTO,
      field('into', $._identifier),
      optional($.with_pointer),
    ),

    string_item: $ => choice(
      $._x,
      seq($.DELIMITED, optional($._BY), choice($.SIZE, $._x)),
    ),

    with_pointer: $ => seq(
      optional($._WITH),
      $._POINTER,
      $._identifier
    ),

    subtract_statement: $ => seq(
      $.SUBTRACT,
      $._subtract_body,
    ),

    _subtract_body: $ => choice(
      seq(
        field('x', repeat1($._x)),
        $._FROM,
        choice(
          field('from', repeat1($.arithmetic_x)),
          seq(
            field('from', $._x),
            $._GIVING,
            field('giving', repeat1($.arithmetic_x))
          )
        )
      ),
      seq(
        $.CORRESPONDING,
        field('x', $._identifier),
        $._FROM,
        field('from', $._identifier),
        optional($.ROUNDED),
      )
    ),

    unstring_statement: $ => seq(
      $._UNSTRING,
      field('x', $._identifier),
      optional($.unstring_delimited),
      $._INTO,
      repeat1($.unstring_into_item),
      optional($.with_pointer),
      optional($.unstring_tallying),
    ),

    unstring_delimited: $ => seq(
      $._DELIMITED,
      optional($._BY),
      repeat1($.unstring_delimited_item)
    ),

    unstring_delimited_item: $ => seq(
      optional($.ALL),
      field('x', $._simple_value)
    ),

    unstring_into_item: $ => seq(
      field('x', $._identifier),
      optional(seq(
        $._DELIMITER,
        optional($._IN),
        field('delimiter', $._identifier)
      )),
      optional(seq(
        $._COUNT,
        optional($._IN),
        field('count', $._identifier)
      )),
    ),

    unstring_tallying: $ => seq(
      $._TALLYING,
      optional($._IN),
      field('tallying', $._identifier)
    ),

    use_statement: $ => choice(
      $._use_exception,
      $._use_debugging,
      $._use_reporing
    ),

    _use_exception: $ => seq(
      $._USE,
      optional($.GLOBAL),
      optional($._AFTER),
      optional($._STANDARD),
      choice($._EXCEPTION, $._ERROR),
      optional($._PROCEDURE),
      optional($._ON),
      choice(
        field('file_name_list', repeat1($.WORD)),
        $.INPUT,
        $.OUTPUT,
        $.I_O,
        $.EXTEND
      )
    ),

    _use_debugging: $ => seq(
      $._USE,
      optional($._FOR),
      $._DEBUGGING,
      optional($._ON),
      choice(
        field('procedure_name', $.label),
        seq($.ALL, $.PROCEDURE)
      ),
    ),

    _use_reporing: $ => seq(
      $._USE,
      optional($.GLOBAL),
      $._BEFORE,
      $._REPORTING,
      $._identifier
    ),

    write_statement: $ => seq(
      $._write_statement_header,
    ),

    _write_statement_header: $ => seq(
      $.WRITE,
      field('record_name', $.qualified_word),
      field('from', optional(seq($._FROM, $._id_or_lit))),
      field('lock', optional(choice($.write_lock, $.write_no_lock))),
      field('option', optional($.write_option))
    ),

    write_lock: $ => seq(
      optional($._WITH),
      $._LOCK
    ),

    write_no_lock: $ => seq(
      optional($._WITH),
      $._NO,
      $._LOCK
    ),

    write_option: $ => choice(
      seq(
        choice($.BEFORE, $.AFTER),
        optional($._ADVANCING),
        $._num_or_id_or_lit,
        optional(choice($._LINE, $._LINES))
      ),
      seq(
        choice($.BEFORE, $.AFTER),
        optional($._ADVANCING),
        $.MNEMONIC_NAME,
      ),
      seq(
        choice($.BEFORE, $.AFTER),
        optional($._ADVANCING),
        $.PAGE,
      ),
    ),

    _write_handler: $ => nonempty(
      nonempty($.eop, $.not_eop),
      nonempty($.invalid_key, $.not_invalid_key),
    ),

    eop: $ => prec(1, seq(
      optional($._AT),
      choice($._EOP, $._END_OF_PAGE),
    )),

    not_eop: $ => seq(
      optional(seq(
        $._NOT,
        $._AT)),
      choice($._EOP, $._END_OF_PAGE),
    ),

    invalid_key: $ => seq(
      $._INVALID,
      optional($._KEY),
    ),

    not_invalid_key: $ => seq(
      $._NOT,
      $._INVALID,
      optional($._KEY),
    ),

    next_sentence_statement: $ => seq($._NEXT, $._SENTENCE),

    _simple_value: $ => choice(
      $._identifier,
      $._basic_literal
    ),

    _basic_literal: $ => prec(1, sepBy(
      $._basic_value,
      '&'
    )),

    _basic_value: $ => choice(
      $._LITERAL,
      $.SPACE,
      $.ZERO,
      $.ZEROS,
      $.QUOTE,
      $.HIGH_VALUE,
      $.LOW_VALUE,
      $.TOK_NULL
    ),

    _literal: $ => choice(
      $._basic_literal,
      seq($.ALL, $._basic_value)
    ),

    function_: $ => prec.right(seq($._FUNCTION, choice(
      seq($.CURRENT_DATE_FUNC, optional($.func_refmod)),
      seq($.WHEN_COMPILED_FUNC, optional($.func_refmod)),
      seq($.UPPER_CASE_FUNC, '(', $._exp, ')', optional($.func_refmod)),
      seq($.LOWER_CASE_FUNC, '(', $._exp, ')', optional($.func_refmod)),
      seq($.REVERSE_FUNC, '(', $._exp, ')', optional($.func_refmod)),
      seq($.CONCATENATE_FUNC, '(', $._exp_list, ')', optional($.func_refmod)),
      seq($.SUBSTITUTE_FUNC, '(', $._exp_list, ')', optional($.func_refmod)),
      seq($.SUBSTITUTE_CASE_FUNC, '(', $._exp_list, ')', optional($.func_refmod)),
      seq($.TRIM_FUNCTION, '(', $._trim_args, ')', optional($.func_refmod)),
      seq($.NUMVALC_FUNC, '(', $._numvalc_args, ')'),
      seq($.LOCALE_DT_FUNC, '(', $._locale_dt_args, ')', optional($.func_refmod)),
      seq($.WORD, optional($.func_args)),
    ))),

    func_refmod: $ => choice(
      seq('(', $._exp, ':', ')'),
      seq('(', $._exp, ':', $._exp, ')'),
    ),

    _trim_args: $ => seq(
      $._exp,
      optional(
        choice($._LEADING, $._TRAILING)
      )
    ),

    _numvalc_args: $ => repeat1($._exp),

    _locale_dt_args: $ => seq(
      $._exp,
      optional($.qualified_word)
    ),

    linage_counter: $ => seq(
      $._LINAGE_COUNTER,
      optional(seq($._in_of, $._WORD))
    ),

    func_args: $ => seq(
      '(', $._exp_list, ')'
    ),

    _LITERAL: $ => choice(
      $.number,
      $._string,
    ),

    //todo
    number: $ => choice($.integer, $.decimal),
    integer: $ => /[+-]?[0-9,]+/,
    decimal: $ => /[+-]?[0-9]*\.[0-9]+/,
    _string: $ => choice(
      $.string,
      $.x_string,
      $.n_string,
      $.h_string
    ),

    string: $ => choice(
      /('[^'\n]*')+/,
      /("[^"\n]*")+/,
      $._multiline_string,
    ),

    x_string: $ => choice(
      /X'[^'\n]*'/,
      /X"[^"\n]*"/,
    ),

    h_string: $ => choice(
      /H'[^'\n]*'/,
      /H"[^"\n]*"/,
    ),

    n_string: $ => choice(
      /N'[^'\n]*'/,
      /N"[^"\n]*"/,
      /NC'[^'\n]*'/,
      /NC"[^"\n]*"/,
      /ND'[^'\n]*'/,
      /ND"[^"\n]*"/,
      /NX'[^'\n]*'/,
      /NX"[^"\n]*"/,
    ),

    _WRITE: $ => /[wW][rR][iI][tT][eE]/,
    _ACCEPT: $ => /[aA][cC][cC][eE][pP][tT]/,
    _ACCESS: $ => /[aA][cC][cC][eE][sS][sS]/,
    _ADD: $ => /[aA][dD][dD]/,
    _ADDRESS: $ => /[aA][dD][dD][rR][eE][sS][sS]/,
    _ADVANCING: $ => /[aA][dD][vV][aA][nN][cC][iI][nN][gG]/,
    _AFTER: $ => /[aA][fF][tT][eE][rR]/,
    _ALL: $ => /[aA][lL][lL]/,
    _ALLOCATE: $ => /[aA][lL][lL][oO][cC][aA][tT][eE]/,
    _ALPHABET: $ => /[aA][lL][pP][hH][aA][bB][eE][tT]/,
    _ALPHABETIC: $ => /[aA][lL][pP][hH][aA][bB][eE][tT][iI][cC]/,
    _ALPHABETIC_LOWER: $ => /[aA][lL][pP][hH][aA][bB][eE][tT][iI][cC]-[lL][oO][wW][eE][rR]/,
    _ALPHABETIC_UPPER: $ => /[aA][lL][pP][hH][aA][bB][eE][tT][iI][cC]-[uU][pP][pP][eE][rR]/,
    _ALPHANUMERIC: $ => /[aA][lL][pP][hH][aA][nN][uU][mM][eE][rR][iI][cC]/,
    _ALPHANUMERIC_EDITED: $ => /[aA][lL][pP][hH][aA][nN][uU][mM][eE][rR][iI][cC]-[eE][dD][iI][tT][eE][dD]/,
    _ALSO: $ => /[aA][lL][sS][oO]/,
    _ALTER: $ => /[aA][lL][tT][eE][rR]/,
    _ALTERNATE: $ => /[aA][lL][tT][eE][rR][nN][aA][tT][eE]/,
    _AND: $ => /[aA][nN][dD]/,
    _ANY: $ => /[aA][nN][yY]/,
    _APPLY: $ => /[aA][pP][pP][lL][yY]/,
    _ARE: $ => /[aA][rR][eE]/,
    _AREA: $ => /[aA][rR][eE][aA]/,
    _AREAS: $ => /[aA][rR][eE][aA][sS]/,
    _ARGUMENT_NUMBER: $ => /[aA][rR][gG][uU][mM][eE][nN][tT]-[nN][uU][mM][bB][eE][rR]/,
    _ARGUMENT_VALUE: $ => /[aA][rR][gG][uU][mM][eE][nN][tT]-[vV][aA][lL][uU][eE]/,
    _AS: $ => /[aA][sS]/,
    _ASCENDING: $ => /[aA][sS][cC][eE][nN][dD][iI][nN][gG]/,
    _ASSIGN: $ => /[aA][sS][sS][iI][gG][nN]/,
    _AT: $ => /[aA][tT]/,
    _AUTO: $ => /[aA][uU][tT][oO]/,
    _AUTOMATIC: $ => /[aA][uU][tT][oO][mM][aA][tT][iI][cC]/,
    _BACKGROUND_COLOR: $ => /[bB][aA][cC][kK][gG][rR][oO][uU][nN][dD]-[cC][oO][lL][oO][rR]/,
    _BASED: $ => /[bB][aA][sS][eE][dD]/,
    _BEFORE: $ => /[bB][eE][fF][oO][rR][eE]/,
    _BELL: $ => /[bB][eE][lL][lL]/,
    _BINARY: $ => /[bB][iI][nN][aA][rR][yY]/,
    _BINARY_C_LONG: $ => /[bB][iI][nN][aA][rR][yY]-[cC]-[lL][oO][nN][gG]/,
    _BINARY_CHAR: $ => /[bB][iI][nN][aA][rR][yY]-[cC][hH][aA][rR]/,
    _BINARY_DOUBLE: $ => /[bB][iI][nN][aA][rR][yY]-[dD][oO][uU][bB][lL][eE]/,
    _BINARY_LONG: $ => /[bB][iI][nN][aA][rR][yY]-[lL][oO][nN][gG]/,
    _BINARY_SHORT: $ => /[bB][iI][nN][aA][rR][yY]-[sS][hH][oO][rR][tT]/,
    _BLANK: $ => /[bB][lL][aA][nN][kK]/,
    _BLANK_LINE: $ => /[bB][lL][aA][nN][kK]-[lL][iI][nN][eE]/,
    _BLANK_SCREEN: $ => /[bB][lL][aA][nN][kK]-[sS][cC][rR][eE][eE][nN]/,
    _BLINK: $ => /[bB][lL][iI][nN][kK]/,
    _BLOCK: $ => /[bB][lL][oO][cC][kK]/,
    _BOTTOM: $ => /[bB][oO][tT][tT][oO][mM]/,
    _BY: $ => /[bB][yY]/,
    _BYTE_LENGTH: $ => /[bB][yY][tT][eE]-[lL][eE][nN][gG][tT][hH]/,
    _CALL: $ => /[cC][aA][lL][lL]/,
    _CANCEL: $ => /[cC][aA][nN][cC][eE][lL]/,
    _CH: $ => /[cC][hH]/,
    _CHAINING: $ => /[cC][hH][aA][iI][nN][iI][nN][gG]/,
    _CHARACTER: $ => /[cC][hH][aA][rR][aA][cC][tT][eE][rR]/,
    _CHARACTERS: $ => /[cC][hH][aA][rR][aA][cC][tT][eE][rR][sS]/,
    _CLASS: $ => /[cC][lL][aA][sS][sS]/,
    _CLASS_NAME: $ => /[cC][lL][aA][sS][sS]-[nN][aA][mM][eE]/,
    _COPY: $ => /[cC][oO][pP][yY]/,
    _CLOSE: $ => /[cC][lL][oO][sS][eE]/,
    _CLOSE_NOFEED: $ => /[cC][lL][oO][sS][eE]-[nN][oO][fF][eE][eE][dD]/,
    _CODE: $ => /[cC][oO][dD][eE]/,
    _CODE_SET: $ => /[cC][oO][dD][eE]-[sS][eE][tT]/,
    _COLLATING: $ => /[cC][oO][lL][lL][aA][tT][iI][nN][gG]/,
    _COL: $ => /[cC][oO][lL]/,
    _COLS: $ => /[cC][oO][lL][sS]/,
    _COLUMN: $ => /[cC][oO][lL][uU][mM][nN]/,
    _COLUMNS: $ => /[cC][oO][lL][uU][mM][nN][sS]/,
    _COMMA: $ => /[cC][oO][mM][mM][aA]/,
    _COMMAND_LINE: $ => /[cC][oO][mM][mM][aA][nN][dD]-[lL][iI][nN][eE]/,
    _COMMA_DELIM: $ => /,+/,
    _COMMIT: $ => /[cC][oO][mM][mM][iI][tT]/,
    _COMMITMENT_CONTROL: $ => /[cC][oO][mM][mM][iI][tT][mM][eE][nN][tT]-[cC][oO][nN][tT][rR][oO][lL]/,
    _COMMON: $ => /[cC][oO][mM][mM][oO][nN]/,
    _COMP: $ => /[cC][oO][mM][pP]/,
    _COMPUTE: $ => /[cC][oO][mM][pP][uU][tT][eE]/,
    _COMP_1: $ => /[cC][oO][mM][pP]-1/,
    _COMP_2: $ => /[cC][oO][mM][pP]-2/,
    _COMP_3: $ => /[cC][oO][mM][pP]-3/,
    _COMP_4: $ => /[cC][oO][mM][pP]-4/,
    _COMP_5: $ => /[cC][oO][mM][pP]-5/,
    _COMP_X: $ => /[cC][oO][mM][pP]-[xX]/,
    _CONCATENATE_FUNC: $ => /[cC][oO][nN][cC][aA][tT][eE][nN][aA][tT][eE]-[fF][uU][nN][cC]/,
    _CONFIGURATION: $ => /[cC][oO][nN][fF][iI][gG][uU][rR][aA][tT][iI][oO][nN]/,
    _CONSTANT: $ => /[cC][oO][nN][sS][tT][aA][nN][tT]/,
    _CONTAINS: $ => /[cC][oO][nN][tT][aA][iI][nN][sS]/,
    _CONTENT: $ => /[cC][oO][nN][tT][eE][nN][tT]/,
    _CONTINUE: $ => /[cC][oO][nN][tT][iI][nN][uU][eE]/,
    _CONTROL: $ => /[cC][oO][nN][tT][rR][oO][lL]/,
    _CONTROLS: $ => /[cC][oO][nN][tT][rR][oO][lL][sS]/,
    _CONTROL_FOOTING: $ => /[cC][oO][nN][tT][rR][oO][lL]-[fF][oO][oO][tT][iI][nN][gG]/,
    _CONTROL_HEADING: $ => /[cC][oO][nN][tT][rR][oO][lL]-[hH][eE][aA][dD][iI][nN][gG]/,
    _CONVERTING: $ => /[cC][oO][nN][vV][eE][rR][tT][iI][nN][gG]/,
    _CORE_INDEX: $ => /[cC][oO][rR][eE]-[iI][nN][dD][eE][xX]/,
    _CORRESPONDING: $ => /[cC][oO][rR][rR]([eE][sS][pP][oO][nN][dD][iI][nN][gG])?/,
    _COUNT: $ => /[cC][oO][uU][nN][tT]/,
    _CRT: $ => /[cC][rR][tT]/,
    _CURRENCY: $ => /[cC][uU][rR][rR][eE][nN][cC][yY]/,
    _CURRENT_DATE_FUNC: $ => /[cC][uU][rR][rR][eE][nN][tT]-[dD][aA][tT][eE]-[fF][uU][nN][cC]/,
    _CURSOR: $ => /[cC][uU][rR][sS][oO][rR]/,
    _CYCLE: $ => /[cC][yY][cC][lL][eE]/,
    _CYL_OVERFLOW: $ => /[cC][yY][lL]-[oO][vV][eE][rR][fF][lL][oO][wW]/,
    _DATA: $ => /[dD][aA][tT][aA]/,
    _DATE: $ => /[dD][aA][tT][eE]/,
    _DAY: $ => /[dD][aA][yY]/,
    _DAY_OF_WEEK: $ => /[dD][aA][yY]-[oO][fF]-[wW][eE][eE][kK]/,
    _DE: $ => /[dD][eE]/,
    _DEBUGGING: $ => /[dD][eE][bB][uU][gG][gG][iI][nN][gG]/,
    _DECIMAL_POINT: $ => /[dD][eE][cC][iI][mM][aA][lL]-[pP][oO][iI][nN][tT]/,
    _DECLARATIVES: $ => /[dD][eE][cC][lL][aA][rR][aA][tT][iI][vV][eE][sS]/,
    _DEFAULT: $ => /[dD][eE][fF][aA][uU][lL][tT]/,
    _DELETE: $ => /[dD][eE][lL][eE][tT][eE]/,
    _DELIMITED: $ => /[dD][eE][lL][iI][mM][iI][tT][eE][dD]/,
    _DELIMITER: $ => /[dD][eE][lL][iI][mM][iI][tT][eE][rR]/,
    _DEPENDING: $ => /[dD][eE][pP][eE][nN][dD][iI][nN][gG]/,
    _DESCENDING: $ => /[dD][eE][sS][cC][eE][nN][dD][iI][nN][gG]/,
    _DETAIL: $ => /[dD][eE][tT][aA][iI][lL]/,
    _DISK: $ => /[dD][iI][sS][kK]/,
    _DISPLAY: $ => /[dD][iI][sS][pP][lL][aA][yY]/,
    _DIVIDE: $ => /[dD][iI][vV][iI][dD][eE]/,
    _DIVISION: $ => /[dD][iI][vV][iI][sS][iI][oO][nN]/,
    _DOWN: $ => /[dD][oO][wW][nN]/,
    _DUPLICATES: $ => /[dD][uU][pP][lL][iI][cC][aA][tT][eE][sS]/,
    _DYNAMIC: $ => /[dD][yY][nN][aA][mM][iI][cC]/,
    _EBCDIC: $ => /[eE][bB][cC][dD][iI][cC]/,
    _ELSE: $ => /[eE][lL][sS][eE]/,
    _END: $ => /[eE][nN][dD]/,
    _END_ACCEPT: $ => /[eE][nN][dD]-[aA][cC][cC][eE][pP][tT]/,
    _END_ADD: $ => /[eE][nN][dD]-[aA][dD][dD]/,
    _END_CALL: $ => /[eE][nN][dD]-[cC][aA][lL][lL]/,
    _END_COMPUTE: $ => /[eE][nN][dD]-[cC][oO][mM][pP][uU][tT][eE]/,
    _END_DELETE: $ => /[eE][nN][dD]-[dD][eE][lL][eE][tT][eE]/,
    _END_DISPLAY: $ => /[eE][nN][dD]-[dD][iI][sS][pP][lL][aA][yY]/,
    _END_DIVIDE: $ => /[eE][nN][dD]-[dD][iI][vV][iI][dD][eE]/,
    _END_EVALUATE: $ => /[eE][nN][dD]-[eE][vV][aA][lL][uU][aA][tT][eE]/,
    _END_FUNCTION: $ => /[eE][nN][dD]-[fF][uU][nN][cC][tT][iI][oO][nN]/,
    _END_IF: $ => /[eE][nN][dD]-[iI][fF]/,
    _END_MULTIPLY: $ => /[eE][nN][dD]-[mM][uU][lL][tT][iI][pP][lL][yY]/,
    _END_OF_PAGE: $ => /[eE][nN][dD]-[oO][fF]-[pP][aA][gG][eE]/,
    _END_PERFORM: $ => /[eE][nN][dD]-[pP][eE][rR][fF][oO][rR][mM]/,
    _END_PROGRAM: $ => /[eE][nN][dD][ \t]+[pP][rR][oO][gG][rR][aA][mM]/,
    _END_READ: $ => /[eE][nN][dD]-[rR][eE][aA][dD]/,
    _END_RETURN: $ => /[eE][nN][dD]-[rR][eE][tT][uU][rR][nN]/,
    _END_REWRITE: $ => /[eE][nN][dD]-[rR][eE][wW][rR][iI][tT][eE]/,
    _END_SEARCH: $ => /[eE][nN][dD]-[sS][eE][aA][rR][cC][hH]/,
    _END_START: $ => /[eE][nN][dD]-[sS][tT][aA][rR][tT]/,
    _END_STRING: $ => /[eE][nN][dD]-[sS][tT][rR][iI][nN][gG]/,
    _END_SUBTRACT: $ => /[eE][nN][dD]-[sS][uU][bB][tT][rR][aA][cC][tT]/,
    _END_UNSTRING: $ => /[eE][nN][dD]-[uU][nN][sS][tT][rR][iI][nN][gG]/,
    _END_WRITE: $ => /[eE][nN][dD]-[wW][rR][iI][tT][eE]/,
    _ENTRY: $ => /[eE][nN][tT][rR][yY]/,
    _ENVIRONMENT: $ => /[eE][nN][vV][iI][rR][oO][nN][mM][eE][nN][tT]/,
    _ENVIRONMENT_NAME: $ => /[eE][nN][vV][iI][rR][oO][nN][mM][eE][nN][tT]-[nN][aA][mM][eE]/,
    _ENVIRONMENT_VALUE: $ => /[eE][nN][vV][iI][rR][oO][nN][mM][eE][nN][tT]-[vV][aA][lL][uU][eE]/,
    _EOL: $ => /[eE][oO][lL]/,
    _EOP: $ => /([aA][tT][ \t\n]+)?([eE][oO][pP]|[eE][nN][dD]-[oO][fF]-[pP][aA][gG][eE])/,
    _EOS: $ => /[eE][oO][sS]/,
    _EQUAL: $ => /[eE][qQ][uU][aA][lL]/,
    _EQUALS: $ => /[eE][qQ][uU][aA][lL][sS]/,
    _ERASE: $ => /[eE][rR][aA][sS][eE]/,
    _ERROR: $ => /[eE][rR][rR][oO][rR]/,
    _ESCAPE: $ => /[eE][sS][cC][aA][pP][eE]/,
    _EVALUATE: $ => /[eE][vV][aA][lL][uU][aA][tT][eE]/,
    _EVENT_STATUS: $ => /[eE][vV][eE][nN][tT]-[sS][tT][aA][tT][uU][sS]/,
    _EXCEPTION: $ => /[eE][xX][cC][eE][pP][tT][iI][oO][nN]/,
    _EXCLUSIVE: $ => /[eE][xX][cC][lL][uU][sS][iI][vV][eE]/,
    _EXIT: $ => /[eE][xX][iI][tT]/,
    _EXTEND: $ => /[eE][xX][tT][eE][nN][dD]/,
    _EXTERNAL: $ => /[eE][xX][tT][eE][rR][nN][aA][lL]/,
    _FD: $ => /[fF][dD]/,
    _FILE_CONTROL: $ => /[fF][iI][lL][eE]-[cC][oO][nN][tT][rR][oO][lL]/,
    _FILE_ID: $ => /[fF][iI][lL][eE]-[iI][dD]/,
    _FILLER: $ => /[fF][iI][lL][lL][eE][rR]/,
    _FINAL: $ => /[fF][iI][nN][aA][lL]/,
    _FIRST: $ => /[fF][iI][rR][sS][tT]/,
    _FOOTING: $ => /[fF][oO][oO][tT][iI][nN][gG]/,
    _FOR: $ => /[fF][oO][rR]/,
    _FOREGROUND_COLOR: $ => /[fF][oO][rR][eE][gG][rR][oO][uU][nN][dD]-[cC][oO][lL][oO][rR]/,
    _FOREVER: $ => /[fF][oO][rR][eE][vV][eE][rR]/,
    _FORMS_OVERLAY: $ => /[fF][oO][rR][mM][sS]-[oO][vV][eE][rR][lL][aA][yY]/,
    _FREE: $ => /[fF][rR][eE][eE]/,
    _FROM: $ => /[fF][rR][oO][mM]/,
    _FULL: $ => /[fF][uU][lL][lL]/,
    _FUNCTION: $ => /[fF][uU][nN][cC][tT][iI][oO][nN]/,
    _FUNCTION_ID: $ => /[fF][uU][nN][cC][tT][iI][oO][nN]-[iI][dD]/,
    _FUNCTION_NAME: $ => /[fF][uU][nN][cC][tT][iI][oO][nN]-[nN][aA][mM][eE]/,
    _GE: $ => /[gG][eE]/,
    _GENERATE: $ => /[gG][eE][nN][eE][rR][aA][tT][eE]/,
    _GIVING: $ => /[gG][iI][vV][iI][nN][gG]/,
    _GLOBAL: $ => /[gG][lL][oO][bB][aA][lL]/,
    _GO: $ => /[gG][oO]/,
    _GOBACK: $ => /[gG][oO][bB][aA][cC][kK]/,
    _GREATER: $ => /[gG][rR][eE][aA][tT][eE][rR]/,
    _GROUP: $ => /[gG][rR][oO][uU][pP]/,
    _HEADING: $ => /[hH][eE][aA][dD][iI][nN][gG]/,
    _HIGHLIGHT: $ => /[hH][iI][gG][hH][lL][iI][gG][hH][tT]/,
    _HIGH_VALUE: $ => choice(
      'high-value', 'high-Value', 'high-VALUE',
      'High-value', 'High-Value', 'High-VALUE',
      'HIGH-value', 'HIGH-Value', 'HIGH-VALUE',
      'high-values', 'high-Values', 'high-VALUES',
      'High-values', 'High-Values', 'High-VALUES',
      'HIGH-values', 'HIGH-Values', 'HIGH-VALUES',
    ),
    _IDENTIFICATION: $ => /[iI][dD][eE][nN][tT][iI][fF][iI][cC][aA][tT][iI][oO][nN]/,
    _IF: $ => /[iI][fF]/,
    _IGNORE: $ => /[iI][gG][nN][oO][rR][eE]/,
    _IGNORING: $ => /[iI][gG][nN][oO][rR][iI][nN][gG]/,
    _IN: $ => /[iI][nN]/,
    _INDEX: $ => /[iI][nN][dD][eE][xX]/,
    _INDEXED: $ => /[iI][nN][dD][eE][xX][eE][dD]/,
    _INDICATE: $ => /[iI][nN][dD][iI][cC][aA][tT][eE]/,
    _INITIALIZE: $ => /[iI][nN][iI][tT][iI][aA][lL][iI][zZ][eE]/,
    _INITIALIZED: $ => /[iI][nN][iI][tT][iI][aA][lL][iI][zZ][eE][dD]/,
    _INITIATE: $ => /[iI][nN][iI][tT][iI][aA][tT][eE]/,
    _INPUT: $ => /[iI][nN][pP][uU][tT]/,
    _INPUT_OUTPUT: $ => /[iI][nN][pP][uU][tT]-[oO][uU][tT][pP][uU][tT]/,
    _INSPECT: $ => /[iI][nN][sS][pP][eE][cC][tT]/,
    _INTO: $ => /[iI][nN][tT][oO]/,
    _INTRINSIC: $ => /[iI][nN][tT][rR][iI][nN][sS][iI][cC]/,
    _INVALID: $ => /[iI][nN][vV][aA][lL][iI][dD]/,
    _INVALID_KEY: $ => /[iI][nN][vV][aA][lL][iI][dD]-[kK][eE][yY]/,
    _IS: $ => /[iI][sS]/,
    _I_O: $ => /[iI]-[oO]/,
    _I_O_CONTROL: $ => /[iI]-[oO]-[cC][oO][nN][tT][rR][oO][lL]/,
    _JUST: $ => /[jJ][uU][sS][tT]/,
    _JUSTIFIED: $ => /[jJ][uU][sS][tT][iI][fF][iI][eE][dD]/,
    _KEY: $ => /[kK][eE][yY]/,
    _LABEL: $ => /[lL][aA][bB][eE][lL]/,
    _LAST: $ => /[lL][aA][sS][tT]/,
    _LAST_DETAIL: $ => /[lL][aA][sS][tT]-[dD][eE][tT][aA][iI][lL]/,
    _LE: $ => /[lL][eE]/,
    _LEADING: $ => /[lL][eE][aA][dD][iI][nN][gG]/,
    _LEFT: $ => /[lL][eE][fF][tT]/,
    _LENGTH: $ => /[lL][eE][nN][gG][tT][hH]/,
    _LESS: $ => /[lL][eE][sS][sS]/,
    _LEVEL_NUMBER_WORD: $ => /[lL][eE][vV][eE][lL]-[nN][uU][mM][bB][eE][rR]-[wW][oO][rR][dD]/,
    _LEVEL88_NUMBER_WORD: $ => /[lL][eE][vV][eE][lL]88-[nN][uU][mM][bB][eE][rR]-[wW][oO][rR][dD]/,
    _LIMIT: $ => /[lL][iI][mM][iI][tT]/,
    _LIMITS: $ => /[lL][iI][mM][iI][tT][sS]/,
    _LINAGE: $ => /[lL][iI][nN][aA][gG][eE]/,
    _LINAGE_COUNTER: $ => /[lL][iI][nN][aA][gG][eE]-[cC][oO][uU][nN][tT][eE][rR]/,
    _LINE: $ => /[lL][iI][nN][eE]/,
    _LINES: $ => /[lL][iI][nN][eE][sS]/,
    _LINKAGE: $ => /[lL][iI][nN][kK][aA][gG][eE]/,
    _LOCALE: $ => /[lL][oO][cC][aA][lL][eE]/,
    _LOCALE_DT_FUNC: $ => /[lL][oO][cC][aA][lL][eE]-[dD][tT]-[fF][uU][nN][cC]/,
    _LOCAL_STORAGE: $ => /[lL][oO][cC][aA][lL]-[sS][tT][oO][rR][aA][gG][eE]/,
    _LOCK: $ => /[lL][oO][cC][kK]/,
    _LOWER_CASE_FUNC: $ => /[lL][oO][wW][eE][rR]-[cC][aA][sS][eE]-[fF][uU][nN][cC]/,
    _LOWLIGHT: $ => /[lL][oO][wW][lL][iI][gG][hH][tT]/,
    _LOW_VALUE: $ => choice(
      'low-value', 'low-Value', 'low-VALUE',
      'Low-value', 'Low-Value', 'Low-VALUE',
      'LOW-value', 'LOW-Value', 'LOW-VALUE',
      'low-values', 'low-Values', 'low-VALUES',
      'Low-values', 'Low-Values', 'Low-VALUES',
      'LOW-values', 'LOW-Values', 'LOW-VALUES',
    ),
    _MANUAL: $ => /[mM][aA][nN][uU][aA][lL]/,
    _MEMORY: $ => /[mM][eE][mM][oO][rR][yY]/,
    _MERGE: $ => /[mM][eE][rR][gG][eE]/,
    _MINUS: $ => /[mM][iI][nN][uU][sS]/,
    //todo
    _MNEMONIC_NAME: $ => /[sS][wW]\-[0-9]/,
    _MODE: $ => /[mM][oO][dD][eE]/,
    _MOVE: $ => /[mM][oO][vV][eE]/,
    _MULTIPLE: $ => /[mM][uU][lL][tT][iI][pP][lL][eE]/,
    _MULTIPLY: $ => /[mM][uU][lL][tT][iI][pP][lL][yY]/,
    _NATIONAL: $ => /[nN][aA][tT][iI][oO][nN][aA][lL]/,
    _NATIONAL_EDITED: $ => /[nN][aA][tT][iI][oO][nN][aA][lL]-[eE][dD][iI][tT][eE][dD]/,
    _NATIVE: $ => /[nN][aA][tT][iI][vV][eE]/,
    _NEGATIVE: $ => /[nN][eE][gG][aA][tT][iI][vV][eE]/,
    _NEXT: $ => /[nN][eE][xX][tT]/,
    _NEXT_SENTENCE: $ => /[nN][eE][xX][tT]-[sS][eE][nN][tT][eE][nN][cC][eE]/,
    _NO: $ => /[nN][oO]/,
    _NOMINAL: $ => /[nN][oO][mM][iI][nN][aA][lL]/,
    _NOT: $ => /[nN][oO][tT]/,
    _NOT_END: $ => /[nN][oO][tT]-[eE][nN][dD]/,
    _NOT_EOP: $ => /[nN][oO][tT][ \t\n]+([aA][tT][ \t\n]+)?([eE][oO][pP]|[eE][nN][dD]-[oO][fF]-[pP][aA][gG][eE])/,
    _NOT_EXCEPTION: $ => /[nN][oO][tT]-[eE][xX][cC][eE][pP][tT][iI][oO][nN]/,
    _NOT_INVALID_KEY: $ => /[nN][oO][tT]-[iI][nN][vV][aA][lL][iI][dD]-[kK][eE][yY]/,
    _NOT_OVERFLOW: $ => /[nN][oO][tT]-[oO][vV][eE][rR][fF][lL][oO][wW]/,
    _NOT_SIZE_ERROR: $ => /[nN][oO][tT]-[sS][iI][zZ][eE]-[eE][rR][rR][oO][rR]/,
    //_NO_ADVANCING: $ => /[nN][oO]-[aA][dD][vV][aA][nN][cC][iI][nN][gG]/,
    _NUMBER: $ => /[nN][uU][mM][bB][eE][rR]/,
    _NUMBERS: $ => /[nN][uU][mM][bB][eE][rR][sS]/,
    _NUMERIC: $ => /[nN][uU][mM][eE][rR][iI][cC]/,
    _NUMERIC_EDITED: $ => /[nN][uU][mM][eE][rR][iI][cC]-[eE][dD][iI][tT][eE][dD]/,
    _NUMVALC_FUNC: $ => /[nN][uU][mM][vV][aA][lL][cC]-[fF][uU][nN][cC]/,
    _OBJECT_COMPUTER: $ => /[oO][bB][jJ][eE][cC][tT]-[cC][oO][mM][pP][uU][tT][eE][rR]/,
    _OCCURS: $ => /[oO][cC][cC][uU][rR][sS]/,
    _OF: $ => /[oO][fF]/,
    _OFF: $ => /[oO][fF][fF]/,
    _OMITTED: $ => /[oO][mM][iI][tT][tT][eE][dD]/,
    _ON: $ => /[oO][nN]/,
    _ONLY: $ => /[oO][nN][lL][yY]/,
    _OPEN: $ => /[oO][pP][eE][nN]/,
    _OPTIONAL: $ => /[oO][pP][tT][iI][oO][nN][aA][lL]/,
    _OR: $ => /[oO][rR]/,
    _ORDER: $ => /[oO][rR][dD][eE][rR]/,
    _ORGANIZATION: $ => /[oO][rR][gG][aA][nN][iI][zZ][aA][tT][iI][oO][nN]/,
    _OTHER: $ => /[oO][tT][hH][eE][rR]/,
    _OUTPUT: $ => /[oO][uU][tT][pP][uU][tT]/,
    _OVERFLOW: $ => /[oO][vV][eE][rR][fF][lL][oO][wW]/,
    _OVERLINE: $ => /[oO][vV][eE][rR][lL][iI][nN][eE]/,
    _PACKED_DECIMAL: $ => /[pP][aA][cC][kK][eE][dD]-[dD][eE][cC][iI][mM][aA][lL]/,
    _PADDING: $ => /[pP][aA][dD][dD][iI][nN][gG]/,
    _PAGE: $ => /[pP][aA][gG][eE]/,
    _PAGE_FOOTING: $ => /[pP][aA][gG][eE]-[fF][oO][oO][tT][iI][nN][gG]/,
    _PAGE_HEADING: $ => /[pP][aA][gG][eE]-[hH][eE][aA][dD][iI][nN][gG]/,
    _PARAGRAPH: $ => /[pP][aA][rR][aA][gG][rR][aA][pP][hH]/,
    _PERFORM: $ => /[pP][eE][rR][fF][oO][rR][mM]/,
    _PIC: $ => /[pP][iI][cC]/,
    _PICTURE: $ => /[pP][iI][cC][tT][uU][rR][eE]/,
    _PLUS: $ => /[pP][lL][uU][sS]/,
    _POINTER: $ => /[pP][oO][iI][nN][tT][eE][rR]/,
    _POSITION: $ => /[pP][oO][sS][iI][tT][iI][oO][nN]/,
    _POSITIVE: $ => /[pP][oO][sS][iI][tT][iI][vV][eE]/,
    _PRESENT: $ => /[pP][rR][eE][sS][eE][nN][tT]/,
    _PREVIOUS: $ => /[pP][rR][eE][vV][iI][oO][uU][sS]/,
    _PRINTER: $ => /[pP][rR][iI][nN][tT][eE][rR]/,
    _PRINTING: $ => /[pP][rR][iI][nN][tT][iI][nN][gG]/,
    _PROCEDURE: $ => /[pP][rR][oO][cC][eE][dD][uU][rR][eE]/,
    _PROCEDURES: $ => /[pP][rR][oO][cC][eE][dD][uU][rR][eE][sS]/,
    _PROCEED: $ => /[pP][rR][oO][cC][eE][eE][dD]/,
    _PROGRAM: $ => /[pP][rR][oO][gG][rR][aA][mM]/,
    _PROGRAM_ID: $ => /[pP][rR][oO][gG][rR][aA][mM]-[iI][dD]/,
    _PROGRAM_NAME: $ => /[pP][rR][oO][gG][rR][aA][mM]-[nN][aA][mM][eE]/,
    _PROGRAM_POINTER: $ => /[pP][rR][oO][gG][rR][aA][mM]-[pP][oO][iI][nN][tT][eE][rR]/,
    _PROMPT: $ => /[pP][rR][oO][mM][pP][tT]/,
    _QUOTE: $ => choice('quote', 'QUOTE', 'Quote', 'quotes', 'QUOTES', 'Quotes'),
    _RANDOM: $ => /[rR][aA][nN][dD][oO][mM]/,
    _RD: $ => /[rR][dD]/,
    _READ: $ => /[rR][eE][aA][dD]/,
    _RECORD: $ => /[rR][eE][cC][oO][rR][dD]/,
    _RECORDING: $ => /[rR][eE][cC][oO][rR][dD][iI][nN][gG]/,
    _RECORDS: $ => /[rR][eE][cC][oO][rR][dD][sS]/,
    _RECURSIVE: $ => /[rR][eE][cC][uU][rR][sS][iI][vV][eE]/,
    _REDEFINES: $ => /[rR][eE][dD][eE][fF][iI][nN][eE][sS]/,
    _REEL: $ => /[rR][eE][eE][lL]/,
    _REFERENCE: $ => /[rR][eE][fF][eE][rR][eE][nN][cC][eE]/,
    _RELATIVE: $ => /[rR][eE][lL][aA][tT][iI][vV][eE]/,
    _RELEASE: $ => /[rR][eE][lL][eE][aA][sS][eE]/,
    _REMAINDER: $ => /[rR][eE][mM][aA][iI][nN][dD][eE][rR]/,
    _REMOVAL: $ => /[rR][eE][mM][oO][vV][aA][lL]/,
    _RENAMES: $ => /[rR][eE][nN][aA][mM][eE][sS]/,
    _REPLACING: $ => /[rR][eE][pP][lL][aA][cC][iI][nN][gG]/,
    _REPORT: $ => /[rR][eE][pP][oO][rR][tT]/,
    _REPORTING: $ => /[rR][eE][pP][oO][rR][tT][iI][nN][gG]/,
    _REPORTS: $ => /[rR][eE][pP][oO][rR][tT][sS]/,
    _REPORT_FOOTING: $ => /[rR][eE][pP][oO][rR][tT]-[fF][oO][oO][tT][iI][nN][gG]/,
    _REPORT_HEADING: $ => /[rR][eE][pP][oO][rR][tT]-[hH][eE][aA][dD][iI][nN][gG]/,
    _REPOSITORY: $ => /[rR][eE][pP][oO][sS][iI][tT][oO][rR][yY]/,
    _REQUIRED: $ => /[rR][eE][qQ][uU][iI][rR][eE][dD]/,
    _RESERVE: $ => /[rR][eE][sS][eE][rR][vV][eE]/,
    _RETURN: $ => /[rR][eE][tT][uU][rR][nN]/,
    _RETURNING: $ => /[rR][eE][tT][uU][rR][nN][iI][nN][gG]/,
    _REVERSE_FUNC: $ => /[rR][eE][vV][eE][rR][sS][eE]-[fF][uU][nN][cC]/,
    _REVERSE_VIDEO: $ => /[rR][eE][vV][eE][rR][sS][eE]-[vV][iI][dD][eE][oO]/,
    _REWIND: $ => /[rR][eE][wW][iI][nN][dD]/,
    _REWRITE: $ => /[rR][eE][wW][rR][iI][tT][eE]/,
    _RIGHT: $ => /[rR][iI][gG][hH][tT]/,
    _ROLLBACK: $ => /[rR][oO][lL][lL][bB][aA][cC][kK]/,
    _ROUNDED: $ => /[rR][oO][uU][nN][dD][eE][dD]/,
    _RUN: $ => /[rR][uU][nN]/,
    _SAME: $ => /[sS][aA][mM][eE]/,
    _SCREEN: $ => /[sS][cC][rR][eE][eE][nN]/,
    _SCREEN_CONTROL: $ => /[sS][cC][rR][eE][eE][nN]-[cC][oO][nN][tT][rR][oO][lL]/,
    _SCROLL: $ => /[sS][cC][rR][oO][lL][lL]/,
    _SD: $ => /[sS][dD]/,
    _SEARCH: $ => /[sS][eE][aA][rR][cC][hH]/,
    _SECTION: $ => /[sS][eE][cC][tT][iI][oO][nN]/,
    _SECURE: $ => /[sS][eE][cC][uU][rR][eE]/,
    _SEGMENT_LIMIT: $ => /[sS][eE][gG][mM][eE][nN][tT]-[lL][iI][mM][iI][tT]/,
    _SELECT: $ => /[sS][eE][lL][eE][cC][tT]/,
    _SEMI_COLON: $ => /;+/,
    _SENTENCE: $ => /[sS][eE][nN][tT][eE][nN][cC][eE]/,
    _SEPARATE: $ => /[sS][eE][pP][aA][rR][aA][tT][eE]/,
    _SEQUENCE: $ => /[sS][eE][qQ][uU][eE][nN][cC][eE]/,
    _SEQUENTIAL: $ => /[sS][eE][qQ][uU][eE][nN][tT][iI][aA][lL]/,
    _SET: $ => /[sS][eE][tT]/,
    _SHARING: $ => /[sS][hH][aA][rR][iI][nN][gG]/,
    _SIGN: $ => /[sS][iI][gG][nN]/,
    _SIGNED: $ => /[sS][iI][gG][nN][eE][dD]/,
    _SIGNED_INT: $ => /[sS][iI][gG][nN][eE][dD]-[iI][nN][tT]/,
    _SIGNED_LONG: $ => /[sS][iI][gG][nN][eE][dD]-[lL][oO][nN][gG]/,
    _SIGNED_SHORT: $ => /[sS][iI][gG][nN][eE][dD]-[sS][hH][oO][rR][tT]/,
    _SIZE: $ => /[sS][iI][zZ][eE]/,
    _SIZE_ERROR: $ => /[sS][iI][zZ][eE]-[eE][rR][rR][oO][rR]/,
    _SORT: $ => /[sS][oO][rR][tT]/,
    _SORT_MERGE: $ => /[sS][oO][rR][tT]-[mM][eE][rR][gG][eE]/,
    _SOURCE: $ => /[sS][oO][uU][rR][cC][eE]/,
    _SOURCE_COMPUTER: $ => /[sS][oO][uU][rR][cC][eE]-[cC][oO][mM][pP][uU][tT][eE][rR]/,
    _SPACE: $ => choice('space', 'SPACE', 'Space', 'spaces', 'SPACES', 'Spaces'),
    _SPECIAL_NAMES: $ => /[sS][pP][eE][cC][iI][aA][lL]-[nN][aA][mM][eE][sS]/,
    _STANDARD: $ => /[sS][tT][aA][nN][dD][aA][rR][dD]/,
    _STANDARD_1: $ => /[sS][tT][aA][nN][dD][aA][rR][dD]-1/,
    _STANDARD_2: $ => /[sS][tT][aA][nN][dD][aA][rR][dD]-2/,
    _START: $ => /[sS][tT][aA][rR][tT]/,
    _STATUS: $ => /[sS][tT][aA][tT][uU][sS]/,
    _STOP: $ => /[sS][tT][oO][pP]/,
    _STRING: $ => /[sS][tT][rR][iI][nN][gG]/,
    _SUBSTITUTE_FUNC: $ => /[sS][uU][bB][sS][tT][iI][tT][uU][tT][eE]-[fF][uU][nN][cC]/,
    _SUBSTITUTE_CASE_FUNC: $ => /[sS][uU][bB][sS][tT][iI][tT][uU][tT][eE]-[cC][aA][sS][eE]-[fF][uU][nN][cC]/,
    _SUBTRACT: $ => /[sS][uU][bB][tT][rR][aA][cC][tT]/,
    _SUM: $ => /[sS][uU][mM]/,
    _SUPPRESS: $ => /[sS][uU][pP][pP][rR][eE][sS][sS]/,
    _SYMBOLIC: $ => /[sS][yY][mM][bB][oO][lL][iI][cC]/,
    _SYNC: $ => /[sS][yY][nN][cC]/,
    _SYNCHRONIZED: $ => /[sS][yY][nN][cC][hH][rR][oO][nN][iI][zZ][eE][dD]/,
    _TALLYING: $ => /[tT][aA][lL][lL][yY][iI][nN][gG]/,
    _TAPE: $ => /[tT][aA][pP][eE]/,
    _TERMINATE: $ => /[tT][eE][rR][mM][iI][nN][aA][tT][eE]/,
    _TEST: $ => /[tT][eE][sS][tT]/,
    _THAN: $ => /[tT][hH][aA][nN]/,
    _THEN: $ => /[tT][hH][eE][nN]/,
    _THRU: $ => /([tT][hH][rR][uU])|[tT][hH][rR][oO][uU][gG][hH]/,
    _TIME: $ => /[tT][iI][mM][eE]/,
    _TIMES: $ => /[tT][iI][mM][eE][sS]/,
    _TO: $ => /[tT][oO]/,
    _FALSE: $ => /[fF][aA][lL][sS][eE]/,
    _FILE: $ => /[fF][iI][lL][eE]/,
    _INITIAL: $ => /[iI][nN][iI][tT][iI][aA][lL]/,
    _TOK_NULL: $ => choice('null', 'Null', 'NULL'),
    _TRUE: $ => /[tT][rR][uU][eE]/,
    _TOP: $ => /[tT][oO][pP]/,
    _TRACKS: $ => /[tT][rR][aA][cC][kK][sS]/,
    _TRAILING: $ => /[tT][rR][aA][iI][lL][iI][nN][gG]/,
    _TRANSFORM: $ => /[tT][rR][aA][nN][sS][fF][oO][rR][mM]/,
    _TRIM_FUNCTION: $ => /[tT][rR][iI][mM]-[fF][uU][nN][cC][tT][iI][oO][nN]/,
    _TYPE: $ => /[tT][yY][pP][eE]/,
    _UNDERLINE: $ => /[uU][nN][dD][eE][rR][lL][iI][nN][eE]/,
    _UNIT: $ => /[uU][nN][iI][tT]/,
    _UNLOCK: $ => /[uU][nN][lL][oO][cC][kK]/,
    _UNSIGNED: $ => /[uU][nN][sS][iI][gG][nN][eE][dD]/,
    _UNSIGNED_INT: $ => /[uU][nN][sS][iI][gG][nN][eE][dD]-[iI][nN][tT]/,
    _UNSIGNED_LONG: $ => /[uU][nN][sS][iI][gG][nN][eE][dD]-[lL][oO][nN][gG]/,
    _UNSIGNED_SHORT: $ => /[uU][nN][sS][iI][gG][nN][eE][dD]-[sS][hH][oO][rR][tT]/,
    _UNSTRING: $ => /[uU][nN][sS][tT][rR][iI][nN][gG]/,
    _UNTIL: $ => /[uU][nN][tT][iI][lL]/,
    _UP: $ => /[uU][pP]/,
    _UPDATE: $ => /[uU][pP][dD][aA][tT][eE]/,
    _UPON: $ => /[uU][pP][oO][nN]/,
    _UPON_ARGUMENT_NUMBER: $ => /[uU][pP][oO][nN]-[aA][rR][gG][uU][mM][eE][nN][tT]-[nN][uU][mM][bB][eE][rR]/,
    _UPON_COMMAND_LINE: $ => /[uU][pP][oO][nN]-[cC][oO][mM][mM][aA][nN][dD]-[lL][iI][nN][eE]/,
    _UPON_ENVIRONMENT_NAME: $ => /[uU][pP][oO][nN]-[eE][nN][vV][iI][rR][oO][nN][mM][eE][nN][tT]-[nN][aA][mM][eE]/,
    _UPON_ENVIRONMENT_VALUE: $ => /[uU][pP][oO][nN]-[eE][nN][vV][iI][rR][oO][nN][mM][eE][nN][tT]-[vV][aA][lL][uU][eE]/,
    _UPPER_CASE_FUNC: $ => /[uU][pP][pP][eE][rR]-[cC][aA][sS][eE]-[fF][uU][nN][cC]/,
    _USAGE: $ => /[uU][sS][aA][gG][eE]/,
    _USE: $ => /[uU][sS][eE]/,
    _USING: $ => /[uU][sS][iI][nN][gG]/,
    _VALUE: $ => /[vV][aA][lL][uU][eE]/,
    _VALUES: $ => /[vV][aA][lL][uU][eE][sS]/,
    _VARYING: $ => /[vV][aA][rR][yY][iI][nN][gG]/,
    _WAIT: $ => /[wW][aA][iI][tT]/,
    _WHEN: $ => /[wW][hH][eE][nN]/,
    _WHEN_COMPILED_FUNC: $ => /[wW][hH][eE][nN]-[cC][oO][mM][pP][iI][lL][eE][dD]-[fF][uU][nN][cC]/,
    _WHEN_OTHER: $ => /[wW][hH][eE][nN][ \t\n]+[oO][tT][hH][eE][rR]/,
    _WITH: $ => /[wW][iI][tT][hH]/,
    _WORD: $ => /([0-9][a-zA-Z0-9-]*[a-zA-Z][a-zA-Z0-9-]*)|([a-zA-Z][a-zA-Z0-9-]*)/,
    _WORDS: $ => /[wW][oO][rR][dD][sS]/,
    _WORKING_STORAGE: $ => /[wW][oO][rR][kK][iI][nN][gG]-[sS][tT][oO][rR][aA][gG][eE]/,
    _YYYYDDD: $ => /[yY][yY][yY][yY][dD][dD][dD]/,
    _YYYYMMDD: $ => /[yY][yY][yY][yY][mM][mM][dD][dD]/,
    _ZERO: $ => choice('zero', 'ZERO', 'Zero'),
    _ZEROS: $ => choice('zeros', 'ZEROS', 'Zeros', 'zeroes', 'ZEROES', 'Zeroes'),


    ACCEPT: $ => $._ACCEPT,
    //ACCESS: $ => $._ACCESS,
    ADD: $ => $._ADD,
    //ADDRESS: $ => $._ADDRESS,
    ADVANCING: $ => $._ADVANCING,
    AFTER: $ => $._AFTER,
    ALL: $ => $._ALL,
    //ALLOCATE: $ => $._ALLOCATE,
    //ALPHABET: $ => $._ALPHABET,
    ALPHABETIC: $ => $._ALPHABETIC,
    ALPHABETIC_LOWER: $ => $._ALPHABETIC_LOWER,
    ALPHABETIC_UPPER: $ => $._ALPHABETIC_UPPER,
    ALPHANUMERIC: $ => $._ALPHANUMERIC,
    ALPHANUMERIC_EDITED: $ => $._ALPHANUMERIC_EDITED,
    ALSO: $ => $._ALSO,
    //ALTER: $ => $._ALTER,
    //ALTERNATE: $ => $._ALTERNATE,
    AND: $ => $._AND,
    ANY: $ => $._ANY,
    //APPLY: $ => $._APPLY,
    //ARE: $ => $._ARE,
    //AREA: $ => $._AREA,
    ARGUMENT_NUMBER: $ => $._ARGUMENT_NUMBER,
    ARGUMENT_VALUE: $ => $._ARGUMENT_VALUE,
    //AS: $ => $._AS,
    ASCENDING: $ => $._ASCENDING,
    ASSIGN: $ => $._ASSIGN,
    //AT: $ => $._AT,
    AUTO: $ => $._AUTO,
    AUTOMATIC: $ => $._AUTOMATIC,
    BACKGROUND_COLOR: $ => $._BACKGROUND_COLOR,
    //BASED: $ => $._BASED,
    BEFORE: $ => $._BEFORE,
    BELL: $ => $._BELL,
    BINARY: $ => $._BINARY,
    BINARY_C_LONG: $ => $._BINARY_C_LONG,
    BINARY_CHAR: $ => $._BINARY_CHAR,
    BINARY_DOUBLE: $ => $._BINARY_DOUBLE,
    BINARY_LONG: $ => $._BINARY_LONG,
    BINARY_SHORT: $ => $._BINARY_SHORT,
    //BLANK: $ => $._BLANK,
    BLANK_LINE: $ => $._BLANK_LINE,
    BLANK_SCREEN: $ => $._BLANK_SCREEN,
    BLINK: $ => $._BLINK,
    //BLOCK: $ => $._BLOCK,
    //BOTTOM: $ => $._BOTTOM,
    BY: $ => $._BY,
    //BYTE_LENGTH: $ => $._BYTE_LENGTH,
    CALL: $ => $._CALL,
    CANCEL: $ => $._CANCEL,
    //CH: $ => $._CH,
    CHAINING: $ => $._CHAINING,
    //CHARACTER: $ => $._CHARACTER,
    CHARACTERS: $ => $._CHARACTERS,
    //CLASS: $ => $._CLASS,
    CLASS_NAME: $ => $._CLASS_NAME,
    CLOSE: $ => $._CLOSE,
    //CLOSE_NOFEED: $ => $._CLOSE_NOFEED,
    //CODE: $ => $._CODE,
    //CODE_SET: $ => $._CODE_SET,
    //COLLATING: $ => $._COLLATING,
    //COL: $ => $._COL,
    //COLS: $ => $._COLS,
    //COLUMN: $ => $._COLUMN,
    COLUMNS: $ => $._COLUMNS,
    //COMMA: $ => $._COMMA,
    COMMAND_LINE: $ => $._COMMAND_LINE,
    //COMMA_DELIM: $ => $._COMMA_DELIM,
    //COMMIT: $ => $._COMMIT,
    COMMITMENT_CONTROL: $ => $._COMMITMENT_CONTROL,
    //COMMON: $ => $._COMMON,
    COMP: $ => $._COMP,
    COMPUTE: $ => $._COMPUTE,
    COMP_1: $ => $._COMP_1,
    COMP_2: $ => $._COMP_2,
    COMP_3: $ => $._COMP_3,
    COMP_4: $ => $._COMP_4,
    COMP_5: $ => $._COMP_5,
    COMP_X: $ => $._COMP_X,
    CONCATENATE_FUNC: $ => $._CONCATENATE_FUNC,
    CONFIGURATION: $ => $._CONFIGURATION,
    //CONSTANT: $ => $._CONSTANT,
    //CONTAINS: $ => $._CONTAINS,
    //CONTENT: $ => $._CONTENT,
    CONTINUE: $ => $._CONTINUE,
    //CONTROL: $ => $._CONTROL,
    //CONTROLS: $ => $._CONTROLS,
    //CONTROL_FOOTING: $ => $._CONTROL_FOOTING,
    //CONTROL_HEADING: $ => $._CONTROL_HEADING,
    //CONVERTING: $ => $._CONVERTING,
    //CORE_INDEX: $ => $._CORE_INDEX,
    CORRESPONDING: $ => $._CORRESPONDING,
    //COUNT: $ => $._COUNT,
    CRT: $ => $._CRT,
    //CURRENCY: $ => $._CURRENCY,
    CURRENT_DATE_FUNC: $ => $._CURRENT_DATE_FUNC,
    //CURSOR: $ => $._CURSOR,
    CYCLE: $ => $._CYCLE,
    //CYL_OVERFLOW: $ => $._CYL_OVERFLOW,
    DATA: $ => $._DATA,
    DATE: $ => $._DATE,
    DAY: $ => $._DAY,
    DAY_OF_WEEK: $ => $._DAY_OF_WEEK,
    //DE: $ => $._DE,
    //DEBUGGING: $ => $._DEBUGGING,
    //DECIMAL_POINT: $ => $._DECIMAL_POINT,
    //DECLARATIVES: $ => $._DECLARATIVES,
    DEFAULT: $ => $._DEFAULT,
    DELETE: $ => $._DELETE,
    DELIMITED: $ => $._DELIMITED,
    //DELIMITER: $ => $._DELIMITER,
    //DEPENDING: $ => $._DEPENDING,
    DESCENDING: $ => $._DESCENDING,
    //DETAIL: $ => $._DETAIL,
    DISK: $ => $._DISK,
    DISPLAY: $ => $._DISPLAY,
    DIVIDE: $ => $._DIVIDE,
    DIVISION: $ => $._DIVISION,
    DOWN: $ => $._DOWN,
    DUPLICATES: $ => $._DUPLICATES,
    DYNAMIC: $ => $._DYNAMIC,
    EBCDIC: $ => $._EBCDIC,
    ELSE: $ => $._ELSE,
    //END: $ => $._END,
    END_ACCEPT: $ => $._END_ACCEPT,
    END_ADD: $ => $._END_ADD,
    END_CALL: $ => $._END_CALL,
    END_COMPUTE: $ => $._END_COMPUTE,
    END_DELETE: $ => $._END_DELETE,
    END_DISPLAY: $ => $._END_DISPLAY,
    END_DIVIDE: $ => $._END_DIVIDE,
    END_EVALUATE: $ => $._END_EVALUATE,
    END_FUNCTION: $ => $._END_FUNCTION,
    END_IF: $ => $._END_IF,
    END_MULTIPLY: $ => $._END_MULTIPLY,
    //END_OF_PAGE: $ => $._END_OF_PAGE,
    END_PERFORM: $ => $._END_PERFORM,
    //END_PROGRAM: $ => $._END_PROGRAM,
    END_READ: $ => $._END_READ,
    END_RETURN: $ => $._END_RETURN,
    END_REWRITE: $ => $._END_REWRITE,
    END_SEARCH: $ => $._END_SEARCH,
    END_START: $ => $._END_START,
    END_STRING: $ => $._END_STRING,
    END_SUBTRACT: $ => $._END_SUBTRACT,
    END_UNSTRING: $ => $._END_UNSTRING,
    END_WRITE: $ => $._END_WRITE,
    //ENTRY: $ => $._ENTRY,
    ENVIRONMENT: $ => $._ENVIRONMENT,
    //ENVIRONMENT_NAME: $ => $._ENVIRONMENT_NAME,
    ENVIRONMENT_VALUE: $ => $._ENVIRONMENT_VALUE,
    EOL: $ => $._EOL,
    //EOP: $ => $._EOP,
    EOS: $ => $._EOS,
    //EQUAL: $ => $._EQUAL,
    //EQUALS: $ => $._EQUALS,
    ERASE: $ => $._ERASE,
    //ERROR: $ => $._ERROR,
    ESCAPE: $ => $._ESCAPE,
    EVALUATE: $ => $._EVALUATE,
    //EVENT_STATUS: $ => $._EVENT_STATUS,
    EXCEPTION: $ => $._EXCEPTION,
    EXCLUSIVE: $ => $._EXCLUSIVE,
    EXIT: $ => $._EXIT,
    EXTEND: $ => $._EXTEND,
    EXTERNAL: $ => $._EXTERNAL,
    FD: $ => $._FD,
    FILE_CONTROL: $ => $._FILE_CONTROL,
    FILE_ID: $ => $._FILE_ID,
    FILLER: $ => $._FILLER,
    //FINAL: $ => $._FINAL,
    FIRST: $ => $._FIRST,
    //FOOTING: $ => $._FOOTING,
    //FOR: $ => $._FOR,
    FOREGROUND_COLOR: $ => $._FOREGROUND_COLOR,
    FOREVER: $ => $._FOREVER,
    //FORMS_OVERLAY: $ => $._FORMS_OVERLAY,
    //FREE: $ => $._FREE,
    //FROM: $ => $._FROM,
    FULL: $ => $._FULL,
    //FUNCTION: $ => $._FUNCTION,
    //FUNCTION_ID: $ => $._FUNCTION_ID,
    FUNCTION_NAME: $ => $._FUNCTION_NAME,
    //GE: $ => $._GE,
    //GENERATE: $ => $._GENERATE,
    //GIVING: $ => $._GIVING,
    GLOBAL: $ => $._GLOBAL,
    GO: $ => $._GO,
    GOBACK: $ => $._GOBACK,
    //GREATER: $ => $._GREATER,
    //GROUP: $ => $._GROUP,
    //HEADING: $ => $._HEADING,
    HIGHLIGHT: $ => $._HIGHLIGHT,
    HIGH_VALUE: $ => $._HIGH_VALUE,
    IDENTIFICATION: $ => $._IDENTIFICATION,
    IF: $ => $._IF,
    IGNORE: $ => $._IGNORE,
    //IGNORING: $ => $._IGNORING,
    //IN: $ => $._IN,
    INDEX: $ => $._INDEX,
    INDEXED: $ => $._INDEXED,
    //INDICATE: $ => $._INDICATE,
    INITIALIZE: $ => $._INITIALIZE,
    INITIALIZED: $ => $._INITIALIZED,
    //INITIATE: $ => $._INITIATE,
    INPUT: $ => $._INPUT,
    INPUT_OUTPUT: $ => $._INPUT_OUTPUT,
    INSPECT: $ => $._INSPECT,
    //INTO: $ => $._INTO,
    //INTRINSIC: $ => $._INTRINSIC,
    //INVALID: $ => $._INVALID,
    //INVALID_KEY: $ => $._INVALID_KEY,
    //IS: $ => $._IS,
    I_O: $ => $._I_O,
    //I_O_CONTROL: $ => $._I_O_CONTROL,
    //JUSTIFIED: $ => $._JUSTIFIED,
    KEY: $ => $._KEY,
    //LABEL: $ => $._LABEL,
    //LAST: $ => $._LAST,
    //LAST_DETAIL: $ => $._LAST_DETAIL,
    //LE: $ => $._LE,
    LEADING: $ => $._LEADING,
    LEFT: $ => $._LEFT,
    LENGTH: $ => $._LENGTH,
    //LESS: $ => $._LESS,
    //LEVEL_NUMBER_WORD: $ => $._LEVEL_NUMBER_WORD,
    //LEVEL88_NUMBER_WORD: $ => $._LEVEL88_NUMBER_WORD,
    //LIMIT: $ => $._LIMIT,
    //LIMITS: $ => $._LIMITS,
    //LINAGE: $ => $._LINAGE,
    //LINAGE_COUNTER: $ => $._LINAGE_COUNTER,
    LINE: $ => $._LINE,
    LINES: $ => $._LINES,
    LINKAGE: $ => $._LINKAGE,
    LITERAL: $ => $._LITERAL,
    //LOCALE: $ => $._LOCALE,
    LOCALE_DT_FUNC: $ => $._LOCALE_DT_FUNC,
    LOCAL_STORAGE: $ => $._LOCAL_STORAGE,
    LOCK: $ => $._LOCK,
    LOWER_CASE_FUNC: $ => $._LOWER_CASE_FUNC,
    LOWLIGHT: $ => $._LOWLIGHT,
    LOW_VALUE: $ => $._LOW_VALUE,
    MANUAL: $ => $._MANUAL,
    //MEMORY: $ => $._MEMORY,
    MERGE: $ => $._MERGE,
    //MINUS: $ => $._MINUS,
    MNEMONIC_NAME: $ => $._MNEMONIC_NAME,
    //MODE: $ => $._MODE,
    MOVE: $ => $._MOVE,
    MULTIPLE: $ => $._MULTIPLE,
    MULTIPLY: $ => $._MULTIPLY,
    NATIONAL: $ => $._NATIONAL,
    NATIONAL_EDITED: $ => $._NATIONAL_EDITED,
    NATIVE: $ => $._NATIVE,
    ////NE: $ => $._NE,
    NEGATIVE: $ => $._NEGATIVE,
    NEXT: $ => $._NEXT,
    //NEXT_SENTENCE: $ => $._NEXT_SENTENCE,
    NO: $ => $._NO,
    //NOMINAL: $ => $._NOMINAL,
    NOT: $ => $._NOT,
    //NOT_END: $ => $._NOT_END,
    //NOT_EOP: $ => $._NOT_EOP,
    NOT_EXCEPTION: $ => $._NOT_EXCEPTION,
    //NOT_INVALID_KEY: $ => $._NOT_INVALID_KEY,
    //NOT_OVERFLOW: $ => $._NOT_OVERFLOW,
    //NOT_SIZE_ERROR: $ => $._NOT_SIZE_ERROR,
    //NO_ADVANCING: $ => $._NO_ADVANCING,
    //NUMBER: $ => $._NUMBER,
    //NUMBERS: $ => $._NUMBERS,
    NUMERIC: $ => $._NUMERIC,
    NUMERIC_EDITED: $ => $._NUMERIC_EDITED,
    NUMVALC_FUNC: $ => $._NUMVALC_FUNC,
    //OBJECT_COMPUTER: $ => $._OBJECT_COMPUTER,
    OCCURS: $ => $._OCCURS,
    //OF: $ => $._OF,
    OFF: $ => $._OFF,
    OMITTED: $ => $._OMITTED,
    ON: $ => $._ON,
    ONLY: $ => $._ONLY,
    OPEN: $ => $._OPEN,
    OPTIONAL: $ => $._OPTIONAL,
    OR: $ => $._OR,
    //ORDER: $ => $._ORDER,
    //ORGANIZATION: $ => $._ORGANIZATION,
    //OTHER: $ => $._OTHER,
    OUTPUT: $ => $._OUTPUT,
    //OVERFLOW: $ => $._OVERFLOW,
    OVERLINE: $ => $._OVERLINE,
    PACKED_DECIMAL: $ => $._PACKED_DECIMAL,
    //PADDING: $ => $._PADDING,
    PAGE: $ => $._PAGE,
    //PAGE_FOOTING: $ => $._PAGE_FOOTING,
    //PAGE_HEADING: $ => $._PAGE_HEADING,
    PARAGRAPH: $ => $._PARAGRAPH,
    PERFORM: $ => $._PERFORM,
    PIC: $ => $._PIC,
    PICTURE: $ => $._PICTURE,
    //PLUS: $ => $._PLUS,
    POINTER: $ => $._POINTER,
    POSITION: $ => $._POSITION,
    POSITIVE: $ => $._POSITIVE,
    //PRESENT: $ => $._PRESENT,
    PREVIOUS: $ => $._PREVIOUS,
    PRINTER: $ => $._PRINTER,
    //PRINTING: $ => $._PRINTING,
    PROCEDURE: $ => $._PROCEDURE,
    //PROCEDURES: $ => $._PROCEDURES,
    PROCEED: $ => $._PROCEED,
    PROGRAM: $ => $._PROGRAM,
    //PROGRAM_ID: $ => $._PROGRAM_ID,
    //PROGRAM_NAME: $ => $._PROGRAM_NAME,
    PROGRAM_POINTER: $ => $._PROGRAM_POINTER,
    PROMPT: $ => $._PROMPT,
    QUOTE: $ => $._QUOTE,
    RANDOM: $ => $._RANDOM,
    //RD: $ => $._RD,
    READ: $ => $._READ,
    RECORD: $ => $._RECORD,
    //RECORDING: $ => $._RECORDING,
    //RECORDS: $ => $._RECORDS,
    //RECURSIVE: $ => $._RECURSIVE,
    //REDEFINES: $ => $._REDEFINES,
    REEL: $ => $._REEL,
    REFERENCE: $ => $._REFERENCE,
    RELATIVE: $ => $._RELATIVE,
    //RELEASE: $ => $._RELEASE,
    //REMAINDER: $ => $._REMAINDER,
    REMOVAL: $ => $._REMOVAL,
    //RENAMES: $ => $._RENAMES,
    //REPLACING: $ => $._REPLACING,
    //REPORT: $ => $._REPORT,
    //REPORTING: $ => $._REPORTING,
    //REPORTS: $ => $._REPORTS,
    //REPORT_FOOTING: $ => $._REPORT_FOOTING,
    //REPORT_HEADING: $ => $._REPORT_HEADING,
    //REPOSITORY: $ => $._REPOSITORY,
    REQUIRED: $ => $._REQUIRED,
    //RESERVE: $ => $._RESERVE,
    RETURN: $ => $._RETURN,
    //RETURNING: $ => $._RETURNING,
    REVERSE_FUNC: $ => $._REVERSE_FUNC,
    REVERSE_VIDEO: $ => $._REVERSE_VIDEO,
    REWIND: $ => $._REWIND,
    REWRITE: $ => $._REWRITE,
    RIGHT: $ => $._RIGHT,
    //ROLLBACK: $ => $._ROLLBACK,
    ROUNDED: $ => $._ROUNDED,
    RUN: $ => $._RUN,
    //SAME: $ => $._SAME,
    //SCREEN: $ => $._SCREEN,
    //SCREEN_CONTROL: $ => $._SCREEN_CONTROL,
    SCROLL: $ => $._SCROLL,
    SD: $ => $._SD,
    SEARCH: $ => $._SEARCH,
    SECTION: $ => $._SECTION,
    //SECURE: $ => $._SECURE,
    //SEGMENT_LIMIT: $ => $._SEGMENT_LIMIT,
    SELECT: $ => $._SELECT,
    //SEMI_COLON: $ => $._SEMI_COLON,
    //SENTENCE: $ => $._SENTENCE,
    SEPARATE: $ => $._SEPARATE,
    //SEQUENCE: $ => $._SEQUENCE,
    SEQUENTIAL: $ => $._SEQUENTIAL,
    SET: $ => $._SET,
    //SHARING: $ => $._SHARING,
    //SIGN: $ => $._SIGN,
    SIGNED: $ => $._SIGNED,
    SIGNED_INT: $ => $._SIGNED_INT,
    SIGNED_LONG: $ => $._SIGNED_LONG,
    SIGNED_SHORT: $ => $._SIGNED_SHORT,
    SIZE: $ => $._SIZE,
    //SIZE_ERROR: $ => $._SIZE_ERROR,
    SORT: $ => $._SORT,
    SORT_MERGE: $ => $._SORT_MERGE,
    SOURCE: $ => $._SOURCE,
    //SOURCE_COMPUTER: $ => $._SOURCE_COMPUTER,
    SPACE: $ => $._SPACE,
    //SPECIAL_NAMES: $ => $._SPECIAL_NAMES,
    STANDARD: $ => $._STANDARD,
    STANDARD_1: $ => $._STANDARD_1,
    STANDARD_2: $ => $._STANDARD_2,
    START: $ => $._START,
    //STATUS: $ => $._STATUS,
    STOP: $ => $._STOP,
    STRING: $ => $._STRING,
    SUBSTITUTE_FUNC: $ => $._SUBSTITUTE_FUNC,
    SUBSTITUTE_CASE_FUNC: $ => $._SUBSTITUTE_CASE_FUNC,
    SUBTRACT: $ => $._SUBTRACT,
    //SUM: $ => $._SUM,
    SUPPRESS: $ => $._SUPPRESS,
    //SYMBOLIC: $ => $._SYMBOLIC,
    //SYNCHRONIZED: $ => $._SYNCHRONIZED,
    //TALLYING: $ => $._TALLYING,
    //TAPE: $ => $._TAPE,
    //TERMINATE: $ => $._TERMINATE,
    //TEST: $ => $._TEST,
    //THAN: $ => $._THAN,
    THEN: $ => $._THEN,
    THRU: $ => $._THRU,
    TIME: $ => $._TIME,
    //TIMES: $ => $._TIMES,
    TO: $ => $._TO,
    FALSE: $ => $._FALSE,
    FILE: $ => $._FILE,
    INITIAL: $ => $._INITIAL,
    TOK_NULL: $ => $._TOK_NULL,
    TRUE: $ => $._TRUE,
    //TOP: $ => $._TOP,
    //TRACKS: $ => $._TRACKS,
    TRAILING: $ => $._TRAILING,
    //TRANSFORM: $ => $._TRANSFORM,
    TRIM_FUNCTION: $ => $._TRIM_FUNCTION,
    //TYPE: $ => $._TYPE,
    UNDERLINE: $ => $._UNDERLINE,
    UNIT: $ => $._UNIT,
    //UNLOCK: $ => $._UNLOCK,
    UNSIGNED: $ => $._UNSIGNED,
    UNSIGNED_INT: $ => $._UNSIGNED_INT,
    UNSIGNED_LONG: $ => $._UNSIGNED_LONG,
    UNSIGNED_SHORT: $ => $._UNSIGNED_SHORT,
    UNSTRING: $ => $._UNSTRING,
    //UNTIL: $ => $._UNTIL,
    UP: $ => $._UP,
    UPDATE: $ => $._UPDATE,
    UPON: $ => $._UPON,
    //UPON_ARGUMENT_NUMBER: $ => $._UPON_ARGUMENT_NUMBER,
    //UPON_COMMAND_LINE: $ => $._UPON_COMMAND_LINE,
    //UPON_ENVIRONMENT_NAME: $ => $._UPON_ENVIRONMENT_NAME,
    //UPON_ENVIRONMENT_VALUE: $ => $._UPON_ENVIRONMENT_VALUE,
    UPPER_CASE_FUNC: $ => $._UPPER_CASE_FUNC,
    //USAGE: $ => $._USAGE,
    //USE: $ => $._USE,
    USING: $ => $._USING,
    VALUE: $ => $._VALUE,
    VARYING: $ => $._VARYING,
    WAIT: $ => $._WAIT,
    WHEN: $ => $._WHEN,
    WHEN_COMPILED_FUNC: $ => $._WHEN_COMPILED_FUNC,
    WHEN_OTHER: $ => $._WHEN_OTHER,
    //WITH: $ => $._WITH,
    WORD: $ => $._WORD,
    WORDS: $ => $._WORDS,
    WORKING_STORAGE: $ => $._WORKING_STORAGE,
    WRITE: $ => $._WRITE,
    YYYYDDD: $ => $._YYYYDDD,
    YYYYMMDD: $ => $._YYYYMMDD,
    ZERO: $ => $._ZERO,
    ZEROS: $ => $._ZEROS,

    _DATE_WRITTEN: $ => /[dD][aA][tT][eE]-[wW][rR][iI][tT][tT][eE][nN]/,
    _DATE_COMPILED: $ => /[dD][aA][tT][eE]-[cC][oO][mM][pP][iI][lL][eE][dD]/,
    _SECURITY: $ => /[sS][eE][cC][uU][rR][iI][tT][yY]/,
    _INSTALLATION: $ => /[iI][nN][sS][tT][aA][lL][lL][aA][tT][iI][oO][nN]/,
    _AUTHOR: $ => /[aA][uU][tT][hH][oO][rR]/,

    COMPUTATIONAL: $ => $._COMPUTATIONAL,
    _COMPUTATIONAL: $ => /[cC][oO][mM][pP][uU][tT][aA][tT][iI][oO][nN][aA][lL]/,
    _NOT_EQUAL: $ => /(!=)|([nN][oO][tT][ \t]+(([eE][qQ][uU][aA][lL])|=))/,
    _NOT_LESS: $ => /([nN][oO][tT][ \t]+(<|[lL][eE][sS][sS]))/,
    _NOT_GREATER: $ => /([nN][oO][tT][ \t]+(>|[gG][rR][eE][aA][tT][eE][rR]))/,

    NOT_OMITTED: $ => /[nN][oO][tT][ \t]+[oO][mM][iI][tT][tT][eE][dD]/,
    NOT_NUMERIC: $ => /[nN][oO][tT][ \t]+[nN][uU][mM][eE][rR][iI][cC]/,
    NOT_ALPHABETIC: $ => /[nN][oO][tT][ \t]+[aA][lL][pP][hH][aA][bB][eE][tT][iI][cC]/,
    NOT_ALPHABETIC_LOWER: $ => /[nN][oO][tT][ \t]+[aA][lL][pP][hH][aA][bB][eE][tT][iI][cC]-[lL][oO][wW][eE][rR]/,
    NOT_ALPHABETIC_UPPER: $ => /[nN][oO][tT][ \t]+[aA][lL][pP][hH][aA][bB][eE][tT][iI][cC]-[uU][pP][pP][eE][rR]/,
    NOT_CLASS_NAME: $ => /[nN][oO][tT][ \t]+[cC][lL][aA][sS][sS]-[nN][aA][mM][eE]/,
    NOT_POSITIVE: $ => /[nN][oO][tT][ \t]+[pP][oO][sS][iI][tT][iI][vV][eE]/,
    NOT_NEGATIVE: $ => /[nN][oO][tT][ \t]+[nN][eE][gG][aA][tT][iI][vV][eE]/,
    NOT_ZERO: $ => /[nN][oO][tT][ \t]+[zZ][eE][rR][oO]/,
    AND_LT: $ => /[aA][nN][dD][ \t]+(<|[lL][eE][sS][sS][ \t]+[tT][hH][aA][nN])/,
    AND_LE: $ => /[aA][nN][dD][ \t]+(<=|[nN][oO][tT][ \t]+(>|[gG][rR][eE][aA][tT][eE][rR][ \t]+[tT][hH][aA][nN]))/,
    AND_GT: $ => /[aA][nN][dD][ \t]+(>|[gG][rR][eE][aA][tT][eE][rR][ \t]+[tT][hH][aA][nN])/,
    AND_GE: $ => /[aA][nN][dD][ \t]+(>=|[nN][oO][tT][ \t]+(<|[lL][eE][sS][sS][ \t]+[tT][hH][aA][nN]))/,
    AND_EQ: $ => /[aA][nN][dD][ \t]+(=|[eE][qQ][uU][aA][lL]([ \t]+[tT][oO])?)/,
    AND_NE: $ => /[aA][nN][dD][ \t]+(!=|[nN][oO][tT][ \t]+[eE][qQ][uU][aA][lL]([ \t]+[tT][oO])?)/,
    OR_LT: $ => /[oO][rR][ \t]+(<|[lL][eE][sS][sS][ \t]+[tT][hH][aA][nN])/,
    OR_LE: $ => /[oO][rR][ \t]+(<=|[nN][oO][tT][ \t]+(>|[gG][rR][eE][aA][tT][eE][rR][ \t]+[tT][hH][aA][nN]))/,
    OR_GT: $ => /[oO][rR][ \t]+(>|[gG][rR][eE][aA][tT][eE][rR][ \t]+[tT][hH][aA][nN])/,
    OR_GE: $ => /[oO][rR][ \t]+(>=|[nN][oO][tT][ \t]+(<|[lL][eE][sS][sS][ \t]+[tT][hH][aA][nN]))/,
    OR_EQ: $ => /[oO][rR][ \t]+(=|[eE][qQ][uU][aA][lL]([ \t]+[tT][oO])?)/,
    OR_NE: $ => /[oO][rR][ \t]+(!=|[nN][oO][tT][ \t]+[eE][qQ][uU][aA][lL]([ \t]+[tT][oO])?)/,
  }
});
