/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// Simple x86 assembly grammar with external scanner for line handling

module.exports = grammar({
  name: "x86asm",

  externals: $ => [
    $._newline,
    $.line_comment,
  ],

  extras: $ => [/[ \t]/],

  inline: $ => [$._operand_item],

  conflicts: $ => [],

  rules: {
    source_file: $ => repeat(choice(
      $.label_definition,
      $.directive,
      $.instruction,
      $.line_comment,
      $._newline,
    )),

    // label: [instruction]
    // Use a single token for label + colon to avoid ambiguity with mnemonic
    label_definition: $ => prec.right(seq(
      field("name", alias($.label_colon, $.label)),
      optional(choice($.directive, $.instruction)),
    )),

    // Label must end with colon - this makes it unambiguous with mnemonic
    label_colon: $ => /[a-zA-Z_.][a-zA-Z0-9_$.]*:/,

    // .directive args
    directive: $ => prec.right(seq(
      $.directive_name,
      optional($.arguments),
    )),

    directive_name: $ => /\.[a-zA-Z_][a-zA-Z0-9_]*/,

    // mnemonic operands
    instruction: $ => seq(
      optional($.prefix),
      $.mnemonic,
      optional($.operands),
    ),

    prefix: $ => token(choice(
      /lock/i, /rep/i, /repe/i, /repz/i, /repne/i, /repnz/i,
      /rex[.a-z]*/i,
    )),

    // Just an identifier - highlights.scm can color known mnemonics
    mnemonic: $ => /[a-zA-Z][a-zA-Z0-9]*/,

    arguments: $ => seq($._argument, repeat(seq(",", $._argument))),

    _argument: $ => choice(
      $.string,
      $.number,
      $.identifier,
      $.directive_name,  // .text, .data, etc. as arguments to directives
    ),

    operands: $ => seq($._operand_item, repeat(seq(",", $._operand_item))),

    _operand_item: $ => choice(
      $.register,
      $.memory_operand,
      $.immediate,
      $.identifier,
    ),

    register: $ => token(prec(2, choice(
      // General purpose 64/32/16/8
      /[re]?ax/i, /[re]?bx/i, /[re]?cx/i, /[re]?dx/i,
      /[re]?si/i, /[re]?di/i, /[re]?sp/i, /[re]?bp/i,
      /r[89][dwb]?/i, /r1[0-5][dwb]?/i,
      /[abcd]l/i, /[abcd]h/i, /sil/i, /dil/i, /spl/i, /bpl/i,
      // Segment
      /[cdefgs]s/i,
      // IP/flags
      /[re]?ip/i, /[re]?flags/i,
      // FPU
      /st\([0-7]\)/i, /st/i,
      // MMX/SSE/AVX
      /[xyz]mm[0-9]/i, /[xyz]mm1[0-5]/i, /[xyz]mm2[0-9]/i, /[xyz]mm3[01]/i,
      /mm[0-7]/i,
      // Mask/control/debug
      /k[0-7]/i, /cr[0-9]/i, /dr[0-7]/i,
    ))),

    memory_operand: $ => seq(
      optional($.ptr_size),
      "[",
      $.memory_expression,
      "]",
    ),

    ptr_size: $ => seq(
      choice("byte", "word", "dword", "qword", "xmmword", "ymmword", "zmmword", "oword", "tword"),
      optional("ptr"),
    ),

    memory_expression: $ => choice(
      $._mem_atom,
      $.binary_expression,
    ),

    binary_expression: $ => prec.left(1, seq(
      $._mem_atom,
      choice("+", "-", "*"),
      $.memory_expression,
    )),

    _mem_atom: $ => choice(
      $.register,
      $.number,
      $.identifier,
    ),

    immediate: $ => seq(optional("$"), $.number),

    number: $ => token(choice(
      /0[xX][0-9a-fA-F]+/,          // 0xDEAD
      /[0-9][0-9a-fA-F]*[hH]/,      // 0DEADh
      /-?[0-9]+/,                   // 123, -45
      /0[bB][01]+/,                 // 0b1010
      /[01]+[bB]/,                  // 1010b
    )),

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_$.]*/,

    string: $ => choice(
      /"[^"]*"/,
      /'[^']*'/,
    ),
  },
});
