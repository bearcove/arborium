// Zsh grammar for syntax highlighting
// Parses structure (commands, functions, control flow) while avoiding pathological cases

module.exports = grammar({
  name: 'zsh',

  extras: $ => [
    /\s/,
    $.comment,
  ],

  // No explicit conflicts needed - precedences handle ambiguity

  word: $ => $.word,

  rules: {
    program: $ => repeat($._statement),

    _statement: $ => choice(
      $.function_definition,
      $.if_statement,
      $.for_statement,
      $.while_statement,
      $.case_statement,
      $.pipeline,
      $.list,
      $.compound_command,
      $.command,
      $.variable_assignment,
      ';',
    ),

    // Comments
    comment: $ => token(prec(-10, seq('#', /.*/))),

    // Function definition
    function_definition: $ => prec(1, choice(
      seq('function', field('name', $.word), optional(seq('(', ')')), field('body', $.compound_command)),
      seq(field('name', $.word), '(', ')', field('body', $.compound_command)),
    )),

    // Compound commands
    compound_command: $ => choice(
      $.brace_group,
      $.subshell,
    ),

    brace_group: $ => seq('{', repeat($._statement), '}'),

    subshell: $ => seq('(', repeat($._statement), ')'),

    // If statement
    if_statement: $ => seq(
      'if',
      field('condition', $._statement),
      'then',
      field('consequence', repeat($._statement)),
      repeat($.elif_clause),
      optional($.else_clause),
      'fi',
    ),

    elif_clause: $ => seq(
      'elif',
      field('condition', $._statement),
      'then',
      field('consequence', repeat($._statement)),
    ),

    else_clause: $ => seq(
      'else',
      field('body', repeat($._statement)),
    ),

    // For loop
    for_statement: $ => seq(
      'for',
      field('variable', $.word),
      optional(seq('in', repeat($._literal))),
      choice(';', '\n'),
      'do',
      field('body', repeat($._statement)),
      'done',
    ),

    // While/until loop
    while_statement: $ => seq(
      choice('while', 'until'),
      field('condition', $._statement),
      'do',
      field('body', repeat($._statement)),
      'done',
    ),

    // Case statement
    case_statement: $ => seq(
      'case',
      field('value', $._literal),
      'in',
      repeat($.case_item),
      'esac',
    ),

    case_item: $ => prec.right(seq(
      optional('('),
      field('pattern', $._literal),
      repeat(seq('|', $._literal)),
      ')',
      repeat($._statement),
      optional(choice(';;', ';&', ';|')),
    )),

    // Pipeline
    pipeline: $ => prec.left(1, seq(
      $._pipeline_command,
      choice('|', '|&'),
      choice($.pipeline, $._pipeline_command),
    )),

    _pipeline_command: $ => choice(
      $.command,
      $.compound_command,
    ),

    // List (&&, ||, &, ;)
    list: $ => prec.left(seq(
      $._statement,
      choice('&&', '||'),
      $._statement,
    )),

    // Simple command - no prefix assignments to avoid ambiguity
    command: $ => prec.left(seq(
      field('name', $.command_name),
      repeat(field('argument', choice($._literal, $.redirection))),
    )),

    command_name: $ => $._literal,

    // Variable assignment - use prec.right to prefer consuming the value
    variable_assignment: $ => prec.right(seq(
      field('name', alias($.variable_name, $.word)),
      choice('=', '+='),
      optional(field('value', choice($._literal, $.array))),
    )),

    variable_name: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // Redirections
    redirection: $ => seq(
      optional($.file_descriptor),
      choice(
        seq(choice('<', '>', '>>', '>&', '<&', '>|', '<>', '&>', '&>>'), $._literal),
        seq('<<<', $._literal),
        seq(choice('<<', '<<-'), $.heredoc_start),
      ),
    ),

    file_descriptor: $ => /[0-9]+/,
    heredoc_start: $ => $.word,

    // Literals (values)
    _literal: $ => choice(
      $.string,
      $.raw_string,
      $.ansii_c_string,
      $.expansion,
      $.simple_expansion,
      $.command_substitution,
      $.process_substitution,
      $.number,
      $.word,
    ),

    // Array
    array: $ => seq('(', repeat($._literal), ')'),

    // Double-quoted string
    string: $ => seq(
      '"',
      repeat(choice(
        $.string_content,
        $.expansion,
        $.simple_expansion,
        $.command_substitution,
        $.escape_sequence,
      )),
      '"',
    ),

    string_content: $ => token(prec(-1, /[^"$\\`]+/)),
    escape_sequence: $ => /\\./,

    // Single-quoted string
    raw_string: $ => /'[^']*'/,

    // ANSI-C quoting $'...'
    ansii_c_string: $ => /\$'([^'\\]|\\.)*'/,

    // Variable expansion
    expansion: $ => /\$\{[^}]+\}/,

    simple_expansion: $ => /\$[a-zA-Z_][a-zA-Z0-9_]*/,

    // Command substitution
    command_substitution: $ => choice(
      seq('$(', repeat($._statement), ')'),
      /`[^`]*`/,
    ),

    // Process substitution
    process_substitution: $ => seq(
      choice('<(', '>('),
      repeat($._statement),
      ')',
    ),

    // Number
    number: $ => /[0-9]+(\.[0-9]+)?/,

    // Word - catch-all for identifiers, paths, etc.
    word: $ => token(prec(-1, /[^\s'"$`(){}<>|&;#\[\]\\=]+/)),

    // Test commands
    test_command: $ => choice(
      seq('[', repeat($._literal), ']'),
      seq('[[', repeat(choice($._literal, $.binary_expression)), ']]'),
    ),

    binary_expression: $ => prec.left(seq(
      $._literal,
      choice('==', '!=', '=~', '-eq', '-ne', '-lt', '-le', '-gt', '-ge'),
      $._literal,
    )),
  },
});
