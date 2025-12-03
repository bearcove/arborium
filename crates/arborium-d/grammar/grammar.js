module.exports = grammar({
  name: 'd',

  word: $ => $.identifier,

  extras: $ => [
    $._white_space,
    $._end_of_line,

    // $.comment, // https://github.com/tree-sitter/tree-sitter/issues/1168
    $.block_comment,
    $.line_comment,
    $.nesting_block_comment,

    $.special_token_sequence,
  ],

  externals: $ => [
    $.nesting_block_comment,
    $.delimited_string,
  ],

  conflicts: $ => [
    // These are probably correct:

    // CaseStatement / CaseRangeStatement disambiguation requires lookahead:
    [$.argument_list, $.first_exp],

    // ---------------------------
    // TODO review grammar:

    [$.qualified_identifier],

    [$.storage_class, $.attribute],
    [$.storage_class, $.enum_declaration],
    [$.storage_class, $.function_attribute_kwd],
    [$.storage_class, $.type_ctor],
    [$.storage_class, $.type_ctor, $.attribute],
    [$.storage_class, $.attribute, $.synchronized_statement],

    [$.attribute, $.static_destructor],
    [$.attribute, $.static_constructor],
    [$.attribute, $.synchronized_statement],

    [$._decl_def, $._declaration],
    [$.alias_assignment, $.qualified_identifier],
    [$.template_instance, $.mixin_qualified_identifier],

    [$.auto_assignment, $.qualified_identifier, $.auto_func_declaration],
    [$.var_declarator, $.func_declarator],

    [$.primary_expression, $.template_instance],

    [$.type_ctor, $.variadic_arguments_attribute],
    [$.in_out, $.variadic_arguments_attribute],
    [$.parameter_attributes],

    [$.storage_class, $.synchronized_statement],

    [$._declaration, $._non_empty_statement_no_case_no_default], // ???

    [$.qualified_identifier, $.template_sequence_parameter],
    [$.qualified_identifier, $.template_type_parameter],
    [$.qualified_identifier, $.template_instance],

    [$.unary_expression, $.parameter],

    [$.new_expression],
    [$.type],

    [$.attribute, $.return_statement],

    [$.primary_expression, $.postblit, $.constructor, $.constructor_template],

    // TODO fix grammar?
    [$._decl_def, $.declaration_statement],
    [$._decl_def, $._non_empty_statement_no_case_no_default],

    [$.foreach_type_list, $.range_foreach], // TODO fix grammar
    [$.foreach_type_attributes],

    [$.parameter, $.template_value_parameter],

    [$.conditional_declaration], // TODO else precedence

    [$.or_or_expression, $.and_and_expression], // TODO precedence
    [$.and_and_expression, $.or_expression], // TODO precedence

    // TODO review these
    [$.or_expression, $.xor_expression],
    [$.xor_expression, $.and_expression],
    [$._cmp_expression, $.rel_expression, $.shift_expression],
    [$._cmp_expression, $.rel_expression],
    [$._cmp_expression, $.identity_expression, $.in_expression],
    [$._cmp_expression, $.equal_expression],
    [$._cmp_expression, $.in_expression],
    [$._cmp_expression, $.identity_expression],
    [$.shift_expression, $.add_expression],
    [$.shift_expression, $.cat_expression],
    [$.add_expression, $.mul_expression],
    [$._maybe_pow_expression, $.postfix_expression],
    [$._maybe_pow_expression, $.index_expression, $.slice_expression],
    [$._maybe_pow_expression, $.pow_expression],
    [$.super_class_or_interface, $.interface],
    [$.postfix_expression, $.template_instance],
    [$.argument_list, $.slice],
    [$.primary_expression, $.symbol_tail],
    [$.type_suffix, $.unary_expression],
    [$.specified_function_body, $.missing_function_body],
    [$.primary_expression, $.destructor],
    [$.declaration_block, $.block_statement],
    [$.parameter_with_member_attributes, $.deallocator],
    [$.conditional_statement],
    [$.static_constructor, $.missing_function_body],
    [$.postblit, $.missing_function_body],
    [$.array_initializer, $.array_literal],
    [$.exp_initializer, $.argument_list],
    [$.array_member_initialization, $.key_expression],
    [$.struct_initializer, $.block_statement],
    [$.mixin_type, $.mixin_expression],
    [$.qualified_identifier, $.symbol_tail],
    [$.asm_rel_exp, $.asm_shift_exp],
    [$.mixin_expression, $.mixin_statement],
    [$.primary_expression, $.synchronized_statement],
    [$.try_statement],
    [$.type_suffix, $.array_literal],
    [$.type_suffix, $.argument_list],
    [$.shared_static_constructor, $.missing_function_body],
    [$.static_destructor, $.missing_function_body],
    [$.parameter, $.template_value_parameter_default],
    [$.unary_expression, $.template_instance],
    [$.rel_expression, $.shift_expression],
    [$.equal_expression, $.shift_expression],
    [$.in_expression, $.shift_expression],
    [$.identity_expression, $.shift_expression],
    [$.cat_expression, $.mul_expression],
    [$.slice],
    [$.asm_log_or_exp, $.asm_log_and_exp],
    [$.asm_log_and_exp, $.asm_or_exp],
    [$.asm_or_exp, $.asm_xor_exp],
    [$.asm_xor_exp, $.asm_and_exp],
    [$.asm_and_exp, $.asm_equal_exp],
    [$.asm_equal_exp, $.asm_rel_exp],
    [$.asm_shift_exp, $.asm_add_exp],
    [$.asm_add_exp, $.asm_mul_exp],
    [$.asm_mul_exp, $.asm_br_exp],
    [$.mixin_declaration, $.mixin_expression, $.mixin_statement],
    [$.shared_static_destructor, $.missing_function_body],
    [$.if_statement],
    [$.mixin_declaration, $.mixin_statement],
    [$.alt_declarator_suffix, $.qualified_identifier],

    [$.storage_classes],
    [$.type_ctors],
    [$.function_contracts],
    [$.decl_defs],
    [$.type_suffixes],
    [$.statement_list_no_case_no_default],
    [$.catches],

    [$.pragma_statement, $.empty_statement],
    [$.empty_declaration, $.empty_statement],

    [$.static_foreach_declaration],
    [$.debug_condition],
    [$.scope_block_statement, $.function_literal_body],
    [$.labeled_statement],
    [$._scope_statement, $.function_literal_body],
    [$._no_scope_statement, $.function_literal_body],
    [$._no_scope_non_empty_statement, $.function_literal_body],
    [$.struct_member_initializer, $.labeled_statement],

    [$.packages],
    [$.qualified_identifier, $.primary_expression],
    [$.basic_type, $.primary_expression],
    [$.qualified_identifier, $.primary_expression, $.symbol_tail],
    [$._maybe_assign_expression, $.assign_expression],
    [$._maybe_conditional_expression, $.or_or_expression],
    [$._maybe_conditional_expression, $.conditional_expression],
    [$._maybe_or_or_expression, $.and_and_expression],
    [$._maybe_and_and_expression, $.or_expression],
    [$._maybe_or_expression, $.xor_expression],
    [$._maybe_xor_expression, $.and_expression],
    [$._maybe_shift_expression, $.add_expression],
    [$._maybe_shift_expression, $.cat_expression],
    [$.exp_initializer, $._maybe_comma_expression],
    [$._maybe_asm_rel_exp, $.asm_shift_exp],
    [$._maybe_asm_exp, $.asm_log_or_exp],
    [$._maybe_asm_exp, $.asm_exp],
    [$._maybe_asm_log_or_exp, $.asm_log_and_exp],
    [$._maybe_asm_log_and_exp, $.asm_or_exp],
    [$._maybe_asm_or_exp, $.asm_xor_exp],
    [$._maybe_asm_xor_exp, $.asm_and_exp],
    [$._maybe_asm_and_exp, $.asm_equal_exp],
    [$._maybe_asm_equal_exp, $.asm_rel_exp],
    [$._maybe_asm_shift_exp, $.asm_add_exp],
    [$._maybe_asm_add_exp, $.asm_mul_exp],
    [$._maybe_asm_mul_exp, $.asm_br_exp],

    [$.storage_class, $.attribute, $.shared_static_constructor, $.shared_static_destructor],
    [$.import_declaration, $.attribute],
    [$._maybe_storage_class, $._maybe_at_attribute],
    [$._maybe_storage_class, $._maybe_attribute],
    [$._module_attribute, $._maybe_attribute],
    [$._module_attribute, $._maybe_at_attribute],
    [$._maybe_basic_type, $.primary_expression],
    [$.type_ctors, $._maybe_in_out],
    [$.enum_members, $._maybe_anonymous_enum_member],
    [$._maybe_basic_type, $.basic_type],
    [$._maybe_attribute, $.pragma_statement],
    [$._decl_def, $._declaration, $._non_empty_statement_no_case_no_default],

    [$.var_declarator_identifier, $.alt_declarator_identifier],
    [$._maybe_add_expression, $.mul_expression],
    [$.primary_expression, $.with_statement, $.symbol_tail],
    [$.alt_declarator_inner, $.qualified_identifier, $.primary_expression],
    [$.alt_declarator_inner, $.primary_expression],
    [$.alt_declarator, $.qualified_identifier, $.primary_expression],
    [$.primary_expression, $.template_value_parameter_default],
    [$.asm_primary_exp],

    [$._maybe_at_attribute, $._function_attribute],
    [$.user_defined_attribute],
    [$.user_defined_attribute, $.template_instance],

    [$.import_declaration, $.storage_class],
    [$.import_declaration, $.storage_class, $.attribute],
    [$.var_declarations],
    [$.func_declaration],

    [$.float_literal, $.primary_expression],
    [$.float_literal, $.template_single_argument],
    [$.token_no_braces, $.float_literal],

    [$.default_statement],
    [$.case_statement],
    [$.case_range_statement],

    [$.storage_class, $.deprecated_attribute],

    [$._cmp_expression, $.shift_expression],

    [$.scope_block_statement, $.specified_function_body],
    [$.specified_function_body],
    [$._scope_statement, $.specified_function_body],
    [$._no_scope_statement, $.specified_function_body],
    [$._no_scope_non_empty_statement, $.specified_function_body],
    [$.missing_function_body, $._function_contract],

    [$.primary_expression, $.parameter],

    [$.alias_assign, $.primary_expression],
    [$.type_suffix],
    [$.member_function_attributes],
    [$.function_attributes],
    [$.qualified_identifier, $.primary_expression, $.template_instance],
    [$.void_initializer, $.fundamental_type],
    [$.type, $.postfix_expression],
    [$._maybe_basic_type, $.basic_type, $.primary_expression],

    [$.primary_expression, $.opcode],
    [$.primary_expression, $.template_instance, $.opcode],
    [$.primary_expression, $.asm_instruction],

    [$.asm_instruction_list, $.gcc_asm_instruction_list],

    [$.qualified_identifier, $.primary_expression, $.opcode],

    // <-- insert here
  ],

  rules: {

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/lex.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/lex.html#SourceFile
    source_file: $ =>
      seq(
        optional(
          choice(
            $.byte_order_mark,
            $.shebang,
          ),
        ),
        optional(
          $.module,
        ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#ByteOrderMark
    byte_order_mark: $ =>
      token(
        // ByteOrderMark
        "\uFEFF",
      ),

    // https://dlang.org/spec/lex.html#Shebang
    shebang: $ =>
      token(
        // Shebang
        seq(
          "#!",
          optional(
            // Characters
            repeat1(
              // Character
              /./,
            ),
          ),
          // EndOfShebang
          choice(
            "\n",
            // EndOfFile
            choice(
              // /$/m,
              // "\0", // https://github.com/tree-sitter/tree-sitter/issues/1167
              "\x1A",
            ),
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#EndOfLine
    _end_of_line: $ =>
      token(
        // EndOfLine
        choice(
          seq(
            "\r",
            optional(
              "\n",
            ),
          ),
          "\n",
          "\u2028",
          "\u2029",
          // EndOfFile
          choice(
            // /$/m,
            // "\0", // https://github.com/tree-sitter/tree-sitter/issues/1167
            "\x1A",
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#WhiteSpace
    _white_space: $ =>
      token(
        // WhiteSpace
        repeat1(
          // Space
          choice(
            " ",
            "\t",
            "\v",
            "\f",
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#Comment
    comment: $ =>
      choice(
        $.block_comment,
        $.line_comment,
        $.nesting_block_comment,
      ),

    // https://dlang.org/spec/lex.html#BlockComment
    block_comment: $ =>
      token(
          // BlockComment
          seq(
            "/*",
	    /[^*]*[*]+(?:[^/*][^*]*[*]+)*/,
            "/",
          ),
      ),

    // https://dlang.org/spec/lex.html#LineComment
    line_comment: $ =>
      token(
          // LineComment
          seq(
            "//",
            optional(
              // Characters
              repeat1(
                // Character
                /./,
              ),
            ),
            // EndOfLine
            choice(
              seq(
                "\r",
                optional(
                  "\n",
                ),
              ),
              "\n",
              "\u2028",
              "\u2029",
              // EndOfFile
              choice(
                // /$/m,
                // "\0", // https://github.com/tree-sitter/tree-sitter/issues/1167
                "\x1A",
              ),
            ),
          ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#TokenNoBraces
    token_no_braces: $ =>
      choice(
        $.identifier,
        $._string_literal,
        $.character_literal,
        $.integer_literal,
        $.float_literal,
        $.keyword,
        "/",
        "/=",
        ".",
        "..",
        "...",
        "&",
        "&=",
        "&&",
        "|",
        "|=",
        "||",
        "-",
        "-=",
        "--",
        "+",
        "+=",
        "++",
        "<",
        "<=",
        "<<",
        "<<=",
        ">",
        ">=",
        ">>=",
        ">>>=",
        ">>",
        ">>>",
        "!",
        "!=",
        "(",
        ")",
        "[",
        "]",
        "?",
        ",",
        ";",
        ":",
        "$",
        "=",
        "==",
        "*",
        "*=",
        "%",
        "%=",
        "^",
        "^=",
        "^^",
        "^^=",
        "~",
        "~=",
        "@",
        "=>",
        "#",
      ),

    // ---

    // https://dlang.org/spec/lex.html#Identifier
    identifier: $ =>
      token(
        // Identifier
        seq(
          // IdentifierStart
          choice(
            "_",
            /[A-Za-z]/,
            /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
          ),
          optional(
            // IdentifierChars
            repeat1(
              // IdentifierChar
              choice(
                // IdentifierStart
                choice(
                  "_",
                  /[A-Za-z]/,
                  /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
                ),
                "0",
                // NonZeroDigit
                choice(
                  "1",
                  "2",
                  "3",
                  "4",
                  "5",
                  "6",
                  "7",
                  "8",
                  "9",
                ),
              ),
            ),
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#StringLiteral
    _string_literal: $ =>
      choice(
        $.wysiwyg_string,
        $.alternate_wysiwyg_string,
        $.double_quoted_string,
        $.hex_string,
        $.delimited_string,
        $.token_string,
      ),

    // ---

    // https://dlang.org/spec/lex.html#WysiwygString
    wysiwyg_string: $ =>
      token(
        // WysiwygString
        seq(
          "r\"",
          optional(
            // WysiwygCharacters
            repeat1(
              // WysiwygCharacter
              choice(
                // Character
                /[^"]/,
               /*
                // EndOfLine
                choice(
                  seq(
                    "\r",
                    optional(
                      "\n",
                    ),
                  ),
                  "\n",
                  "\u2028",
                  "\u2029",
                  // EndOfFile
                  choice(
                    // /$/m,
                    // "\0", // https://github.com/tree-sitter/tree-sitter/issues/1167
                    "\x1A",
                  ),
                ),
               */
              ),
            ),
          ),
          "\"",
          optional(
            // StringPostfix
            choice(
              "c",
              "w",
              "d",
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/lex.html#AlternateWysiwygString
    alternate_wysiwyg_string: $ =>
      token(
        // AlternateWysiwygString
        seq(
          "`",
         /*
          optional(
            // WysiwygCharacters
            repeat1(
              // WysiwygCharacter
              choice(
                // Character
                /[\s\S]/,
                // EndOfLine
                choice(
                  seq(
                    "\r",
                    optional(
                      "\n",
                    ),
                  ),
                  "\n",
                  "\u2028",
                  "\u2029",
                  // EndOfFile
                  choice(
                    // /$/m,
                    // "\0", // https://github.com/tree-sitter/tree-sitter/issues/1167
                    "\x1A",
                  ),
                ),
              ),
            ),
          ),
         */
          /[^`]*/,
          "`",
          optional(
            // StringPostfix
            choice(
              "c",
              "w",
              "d",
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/lex.html#DoubleQuotedString
    double_quoted_string: $ =>
      token(
        // DoubleQuotedString
        seq(
          "\"",
          optional(
            // DoubleQuotedCharacters
            repeat1(
              // DoubleQuotedCharacter
              choice(
                // Character
                /[^"\\]/,
                // EscapeSequence
                choice(
                  "\\'",
                  "\\\"",
                  "\\?",
                  "\\\\",
                  "\\0",
                  "\\a",
                  "\\b",
                  "\\f",
                  "\\n",
                  "\\r",
                  "\\t",
                  "\\v",
                  seq(
                    choice(
                      "\\x",
                      seq(
                        choice(
                          "\\u",
                          seq(
                            "\\U",
                            // HexDigit
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              // HexLetter
                              choice(
                                "a",
                                "b",
                                "c",
                                "d",
                                "e",
                                "f",
                                "A",
                                "B",
                                "C",
                                "D",
                                "E",
                                "F",
                              ),
                            ),
                            // HexDigit
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              // HexLetter
                              choice(
                                "a",
                                "b",
                                "c",
                                "d",
                                "e",
                                "f",
                                "A",
                                "B",
                                "C",
                                "D",
                                "E",
                                "F",
                              ),
                            ),
                            // HexDigit
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              // HexLetter
                              choice(
                                "a",
                                "b",
                                "c",
                                "d",
                                "e",
                                "f",
                                "A",
                                "B",
                                "C",
                                "D",
                                "E",
                                "F",
                              ),
                            ),
                            // HexDigit
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              // HexLetter
                              choice(
                                "a",
                                "b",
                                "c",
                                "d",
                                "e",
                                "f",
                                "A",
                                "B",
                                "C",
                                "D",
                                "E",
                                "F",
                              ),
                            ),
                          ),
                        ),
                        // HexDigit
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          // HexLetter
                          choice(
                            "a",
                            "b",
                            "c",
                            "d",
                            "e",
                            "f",
                            "A",
                            "B",
                            "C",
                            "D",
                            "E",
                            "F",
                          ),
                        ),
                        // HexDigit
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          // HexLetter
                          choice(
                            "a",
                            "b",
                            "c",
                            "d",
                            "e",
                            "f",
                            "A",
                            "B",
                            "C",
                            "D",
                            "E",
                            "F",
                          ),
                        ),
                      ),
                    ),
                    // HexDigit
                    choice(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                      // HexLetter
                      choice(
                        "a",
                        "b",
                        "c",
                        "d",
                        "e",
                        "f",
                        "A",
                        "B",
                        "C",
                        "D",
                        "E",
                        "F",
                      ),
                    ),
                    // HexDigit
                    choice(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                      // HexLetter
                      choice(
                        "a",
                        "b",
                        "c",
                        "d",
                        "e",
                        "f",
                        "A",
                        "B",
                        "C",
                        "D",
                        "E",
                        "F",
                      ),
                    ),
                  ),
                  seq(
                    "\\",
                    choice(
                      seq(
                        // OctalDigit
                        choice(
                          "0",
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                        ),
                        optional(
                          seq(
                            // OctalDigit
                            choice(
                              "0",
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                            ),
                            optional(
                              // OctalDigit
                              choice(
                                "0",
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                              ),
                            ),
                          ),
                        ),
                      ),
                      // NamedCharacterEntity
                      seq(
                        "&",
                        // Identifier
                        seq(
                          // IdentifierStart
                          choice(
                            "_",
                            /[A-Za-z]/,
                            /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
                          ),
                          optional(
                            // IdentifierChars
                            repeat1(
                              // IdentifierChar
                              choice(
                                // IdentifierStart
                                choice(
                                  "_",
                                  /[A-Za-z]/,
                                  /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
                                ),
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                            ),
                          ),
                        ),
                        ";",
                      ),
                    ),
                  ),
                ),
                // EndOfLine
                choice(
                  seq(
                    "\r",
                    optional(
                      "\n",
                    ),
                  ),
                  "\n",
                  "\u2028",
                  "\u2029",
                  // EndOfFile
                  choice(
                    // /$/m,
                    "\0",
                    "\x1A",
                  ),
                ),
              ),
            ),
          ),
          "\"",
          optional(
            // StringPostfix
            choice(
              "c",
              "w",
              "d",
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/lex.html#HexString
    hex_string: $ =>
      token(
        // HexString
        seq(
          "x\"",
          optional(
            // HexStringChars
            repeat1(
              // HexStringChar
              choice(
                // HexDigit
                choice(
                  // DecimalDigit
                  choice(
                    "0",
                    // NonZeroDigit
                    choice(
                      "1",
                      "2",
                      "3",
                      "4",
                      "5",
                      "6",
                      "7",
                      "8",
                      "9",
                    ),
                  ),
                  // HexLetter
                  choice(
                    "a",
                    "b",
                    "c",
                    "d",
                    "e",
                    "f",
                    "A",
                    "B",
                    "C",
                    "D",
                    "E",
                    "F",
                  ),
                ),
                // WhiteSpace
                repeat1(
                  // Space
                  choice(
                    " ",
                    "\t",
                    "\v",
                    "\f",
                  ),
                ),
                // EndOfLine
                choice(
                  seq(
                    "\r",
                    optional(
                      "\n",
                    ),
                  ),
                  "\n",
                  "\u2028",
                  "\u2029",
                  // EndOfFile
                  choice(
                    // /$/m,
                    "\0",
                    "\x1A",
                  ),
                ),
              ),
            ),
          ),
          "\"",
          optional(
            // StringPostfix
            choice(
              "c",
              "w",
              "d",
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/lex.html#DelimitedString
    /*
    delimited_string: $ =>
      token(
        // DelimitedString
        choice(
          seq(
            "q\"",
            // Delimiter
            // Identifier
            seq(
              // IdentifierStart
              choice(
                "_",
                /[A-Za-z]/,
                /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
              ),
              optional(
                // IdentifierChars
                repeat1(
                  // IdentifierChar
                  choice(
                    // IdentifierStart
                    choice(
                      "_",
                      /[A-Za-z]/,
                      /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
                    ),
                    "0",
                    // NonZeroDigit
                    choice(
                      "1",
                      "2",
                      "3",
                      "4",
                      "5",
                      "6",
                      "7",
                      "8",
                      "9",
                    ),
                  ),
                ),
              ),
            ),
            optional(
              // WysiwygCharacters
              repeat1(
                // WysiwygCharacter
                choice(
                  // Character
                  /[\s\S]/,
                  // EndOfLine
                  choice(
                    seq(
                      "\r",
                      optional(
                        "\n",
                      ),
                    ),
                    "\n",
                    "\u2028",
                    "\u2029",
                    // EndOfFile
                    choice(
                      // /$/m,
                      "\0",
                      "\x1A",
                    ),
                  ),
                ),
              ),
            ),
            // MatchingDelimiter
            // Identifier
            seq(
              // IdentifierStart
              choice(
                "_",
                /[A-Za-z]/,
                /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
              ),
              optional(
                // IdentifierChars
                repeat1(
                  // IdentifierChar
                  choice(
                    // IdentifierStart
                    choice(
                      "_",
                      /[A-Za-z]/,
                      /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
                    ),
                    "0",
                    // NonZeroDigit
                    choice(
                      "1",
                      "2",
                      "3",
                      "4",
                      "5",
                      "6",
                      "7",
                      "8",
                      "9",
                    ),
                  ),
                ),
              ),
            ),
            "\"",
          ),
          seq(
            "q\"(",
            optional(
              // ParenDelimitedCharacters
              choice(
                seq(
                  // WysiwygCharacter
                  choice(
                    // Character
                    /[\s\S]/,
                    // EndOfLine
                    choice(
                      seq(
                        "\r",
                        optional(
                          "\n",
                        ),
                      ),
                      "\n",
                      "\u2028",
                      "\u2029",
                      // EndOfFile
                      choice(
                        /$/m,
                        "\0",
                        "\x1A",
                      ),
                    ),
                  ),
                  optional(
                    / * recursion * /,
                  ),
                ),
                seq(
                  "(",
                  optional(
                    / * recursion * /,
                  ),
                  ")",
                ),
              ),
            ),
            ")\"",
          ),
          seq(
            "q\"[",
            optional(
              // BracketDelimitedCharacters
              choice(
                seq(
                  // WysiwygCharacter
                  choice(
                    // Character
                    /[\s\S]/,
                    // EndOfLine
                    choice(
                      seq(
                        "\r",
                        optional(
                          "\n",
                        ),
                      ),
                      "\n",
                      "\u2028",
                      "\u2029",
                      // EndOfFile
                      choice(
                        /$/m,
                        "\0",
                        "\x1A",
                      ),
                    ),
                  ),
                  optional(
                    / * recursion * /,
                  ),
                ),
                seq(
                  "[",
                  optional(
                    / * recursion * /,
                  ),
                  "]",
                ),
              ),
            ),
            "]\"",
          ),
          seq(
            "q\"{",
            optional(
              // BraceDelimitedCharacters
              choice(
                seq(
                  // WysiwygCharacter
                  choice(
                    // Character
                    /[\s\S]/,
                    // EndOfLine
                    choice(
                      seq(
                        "\r",
                        optional(
                          "\n",
                        ),
                      ),
                      "\n",
                      "\u2028",
                      "\u2029",
                      // EndOfFile
                      choice(
                        /$/m,
                        "\0",
                        "\x1A",
                      ),
                    ),
                  ),
                  optional(
                    / * recursion * /,
                  ),
                ),
                seq(
                  "{",
                  optional(
                    / * recursion * /,
                  ),
                  "}",
                ),
              ),
            ),
            "}\"",
          ),
          seq(
            "q\"<",
            optional(
              // AngleDelimitedCharacters
              choice(
                seq(
                  // WysiwygCharacter
                  choice(
                    // Character
                    /[\s\S]/,
                    // EndOfLine
                    choice(
                      seq(
                        "\r",
                        optional(
                          "\n",
                        ),
                      ),
                      "\n",
                      "\u2028",
                      "\u2029",
                      // EndOfFile
                      choice(
                        /$/m,
                        "\0",
                        "\x1A",
                      ),
                    ),
                  ),
                  optional(
                    / * recursion * /,
                  ),
                ),
                seq(
                  "<",
                  optional(
                    / * recursion * /,
                  ),
                  ">",
                ),
              ),
            ),
            ">\"",
          ),
        ),
      ),
    */

    // ---

    // https://dlang.org/spec/lex.html#TokenString
    token_string: $ =>
      seq(
        "q{",
        optional(
          $.token_string_tokens,
        ),
        "}",
      ),

    // https://dlang.org/spec/lex.html#TokenStringTokens
    token_string_tokens: $ =>
      repeat1(
        $._maybe_token_string_token,
      ),

    // https://dlang.org/spec/lex.html#TokenStringToken
    _maybe_token_string_token: $ =>
      choice(
        $.token_no_braces,
        $.token_string_token,
      ),

    token_string_token: $ =>
      seq(
        "{",
        optional(
          $.token_string_tokens,
        ),
        "}",
      ),

    // ---

    // https://dlang.org/spec/lex.html#CharacterLiteral
    character_literal: $ =>
      token(
        // CharacterLiteral
        seq(
          "'",
          // SingleQuotedCharacter
          choice(
            // Character
            /[^'\\]/,
            // EscapeSequence
            choice(
              "\\'",
              "\\\"",
              "\\?",
              "\\\\",
              "\\0",
              "\\a",
              "\\b",
              "\\f",
              "\\n",
              "\\r",
              "\\t",
              "\\v",
              seq(
                choice(
                  "\\x",
                  seq(
                    choice(
                      "\\u",
                      seq(
                        "\\U",
                        // HexDigit
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          // HexLetter
                          choice(
                            "a",
                            "b",
                            "c",
                            "d",
                            "e",
                            "f",
                            "A",
                            "B",
                            "C",
                            "D",
                            "E",
                            "F",
                          ),
                        ),
                        // HexDigit
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          // HexLetter
                          choice(
                            "a",
                            "b",
                            "c",
                            "d",
                            "e",
                            "f",
                            "A",
                            "B",
                            "C",
                            "D",
                            "E",
                            "F",
                          ),
                        ),
                        // HexDigit
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          // HexLetter
                          choice(
                            "a",
                            "b",
                            "c",
                            "d",
                            "e",
                            "f",
                            "A",
                            "B",
                            "C",
                            "D",
                            "E",
                            "F",
                          ),
                        ),
                        // HexDigit
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          // HexLetter
                          choice(
                            "a",
                            "b",
                            "c",
                            "d",
                            "e",
                            "f",
                            "A",
                            "B",
                            "C",
                            "D",
                            "E",
                            "F",
                          ),
                        ),
                      ),
                    ),
                    // HexDigit
                    choice(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                      // HexLetter
                      choice(
                        "a",
                        "b",
                        "c",
                        "d",
                        "e",
                        "f",
                        "A",
                        "B",
                        "C",
                        "D",
                        "E",
                        "F",
                      ),
                    ),
                    // HexDigit
                    choice(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                      // HexLetter
                      choice(
                        "a",
                        "b",
                        "c",
                        "d",
                        "e",
                        "f",
                        "A",
                        "B",
                        "C",
                        "D",
                        "E",
                        "F",
                      ),
                    ),
                  ),
                ),
                // HexDigit
                choice(
                  // DecimalDigit
                  choice(
                    "0",
                    // NonZeroDigit
                    choice(
                      "1",
                      "2",
                      "3",
                      "4",
                      "5",
                      "6",
                      "7",
                      "8",
                      "9",
                    ),
                  ),
                  // HexLetter
                  choice(
                    "a",
                    "b",
                    "c",
                    "d",
                    "e",
                    "f",
                    "A",
                    "B",
                    "C",
                    "D",
                    "E",
                    "F",
                  ),
                ),
                // HexDigit
                choice(
                  // DecimalDigit
                  choice(
                    "0",
                    // NonZeroDigit
                    choice(
                      "1",
                      "2",
                      "3",
                      "4",
                      "5",
                      "6",
                      "7",
                      "8",
                      "9",
                    ),
                  ),
                  // HexLetter
                  choice(
                    "a",
                    "b",
                    "c",
                    "d",
                    "e",
                    "f",
                    "A",
                    "B",
                    "C",
                    "D",
                    "E",
                    "F",
                  ),
                ),
              ),
              seq(
                "\\",
                choice(
                  seq(
                    // OctalDigit
                    choice(
                      "0",
                      "1",
                      "2",
                      "3",
                      "4",
                      "5",
                      "6",
                      "7",
                    ),
                    optional(
                      seq(
                        // OctalDigit
                        choice(
                          "0",
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                        ),
                        optional(
                          // OctalDigit
                          choice(
                            "0",
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                          ),
                        ),
                      ),
                    ),
                  ),
                  // NamedCharacterEntity
                  seq(
                    "&",
                    // Identifier
                    seq(
                      // IdentifierStart
                      choice(
                        "_",
                        /[A-Za-z]/,
                        /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
                      ),
                      optional(
                        // IdentifierChars
                        repeat1(
                          // IdentifierChar
                          choice(
                            // IdentifierStart
                            choice(
                              "_",
                              /[A-Za-z]/,
                              /[\u00aa\u00b5\u00b7\u00ba\u00c0-\u00d6\u00d8-\u00f6\u00f8-\u01f5\u01fa-\u0217\u0250-\u02a8\u02b0-\u02b8\u02bb\u02bd-\u02c1\u02d0-\u02d1\u02e0-\u02e4\u037a\u0386\u0388-\u038a\u038c\u038e-\u03a1\u03a3-\u03ce\u03d0-\u03d6\u03da\u03dc\u03de\u03e0\u03e2-\u03f3\u0401-\u040c\u040e-\u044f\u0451-\u045c\u045e-\u0481\u0490-\u04c4\u04c7-\u04c8\u04cb-\u04cc\u04d0-\u04eb\u04ee-\u04f5\u04f8-\u04f9\u0531-\u0556\u0559\u0561-\u0587\u05b0-\u05b9\u05bb-\u05bd\u05bf\u05c1-\u05c2\u05d0-\u05ea\u05f0-\u05f2\u0621-\u063a\u0640-\u0652\u0660-\u0669\u0670-\u06b7\u06ba-\u06be\u06c0-\u06ce\u06d0-\u06dc\u06e5-\u06e8\u06ea-\u06ed\u06f0-\u06f9\u0901-\u0903\u0905-\u0939\u093d-\u094d\u0950-\u0952\u0958-\u0963\u0966-\u096f\u0981-\u0983\u0985-\u098c\u098f-\u0990\u0993-\u09a8\u09aa-\u09b0\u09b2\u09b6-\u09b9\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09dc-\u09dd\u09df-\u09e3\u09e6-\u09f1\u0a02\u0a05-\u0a0a\u0a0f-\u0a10\u0a13-\u0a28\u0a2a-\u0a30\u0a32-\u0a33\u0a35-\u0a36\u0a38-\u0a39\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a59-\u0a5c\u0a5e\u0a66-\u0a6f\u0a74\u0a81-\u0a83\u0a85-\u0a8b\u0a8d\u0a8f-\u0a91\u0a93-\u0aa8\u0aaa-\u0ab0\u0ab2-\u0ab3\u0ab5-\u0ab9\u0abd-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ad0\u0ae0\u0ae6-\u0aef\u0b01-\u0b03\u0b05-\u0b0c\u0b0f-\u0b10\u0b13-\u0b28\u0b2a-\u0b30\u0b32-\u0b33\u0b36-\u0b39\u0b3d-\u0b43\u0b47-\u0b48\u0b4b-\u0b4d\u0b5c-\u0b5d\u0b5f-\u0b61\u0b66-\u0b6f\u0b82-\u0b83\u0b85-\u0b8a\u0b8e-\u0b90\u0b92-\u0b95\u0b99-\u0b9a\u0b9c\u0b9e-\u0b9f\u0ba3-\u0ba4\u0ba8-\u0baa\u0bae-\u0bb5\u0bb7-\u0bb9\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0be7-\u0bef\u0c01-\u0c03\u0c05-\u0c0c\u0c0e-\u0c10\u0c12-\u0c28\u0c2a-\u0c33\u0c35-\u0c39\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c60-\u0c61\u0c66-\u0c6f\u0c82-\u0c83\u0c85-\u0c8c\u0c8e-\u0c90\u0c92-\u0ca8\u0caa-\u0cb3\u0cb5-\u0cb9\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cde\u0ce0-\u0ce1\u0ce6-\u0cef\u0d02-\u0d03\u0d05-\u0d0c\u0d0e-\u0d10\u0d12-\u0d28\u0d2a-\u0d39\u0d3e-\u0d43\u0d46-\u0d48\u0d4a-\u0d4d\u0d60-\u0d61\u0d66-\u0d6f\u0e01-\u0e3a\u0e40-\u0e5b\u0e81-\u0e82\u0e84\u0e87-\u0e88\u0e8a\u0e8d\u0e94-\u0e97\u0e99-\u0e9f\u0ea1-\u0ea3\u0ea5\u0ea7\u0eaa-\u0eab\u0ead-\u0eae\u0eb0-\u0eb9\u0ebb-\u0ebd\u0ec0-\u0ec4\u0ec6\u0ec8-\u0ecd\u0ed0-\u0ed9\u0edc-\u0edd\u0f00\u0f18-\u0f19\u0f20-\u0f33\u0f35\u0f37\u0f39\u0f3e-\u0f47\u0f49-\u0f69\u0f71-\u0f84\u0f86-\u0f8b\u0f90-\u0f95\u0f97\u0f99-\u0fad\u0fb1-\u0fb7\u0fb9\u10a0-\u10c5\u10d0-\u10f6\u1e00-\u1e9b\u1ea0-\u1ef9\u1f00-\u1f15\u1f18-\u1f1d\u1f20-\u1f45\u1f48-\u1f4d\u1f50-\u1f57\u1f59\u1f5b\u1f5d\u1f5f-\u1f7d\u1f80-\u1fb4\u1fb6-\u1fbc\u1fbe\u1fc2-\u1fc4\u1fc6-\u1fcc\u1fd0-\u1fd3\u1fd6-\u1fdb\u1fe0-\u1fec\u1ff2-\u1ff4\u1ff6-\u1ffc\u203f-\u2040\u207f\u2102\u2107\u210a-\u2113\u2115\u2118-\u211d\u2124\u2126\u2128\u212a-\u2131\u2133-\u2138\u2160-\u2182\u3005-\u3007\u3021-\u3029\u3041-\u3093\u309b-\u309c\u30a1-\u30f6\u30fb-\u30fc\u3105-\u312c\u4e00-\u9fa5\uac00-\ud7a3]/,
                            ),
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                        ),
                      ),
                    ),
                    ";",
                  ),
                ),
              ),
            ),
          ),
          "'",
        ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#IntegerLiteral
    integer_literal: $ =>
      token(
        // IntegerLiteral
        seq(
          // Integer
          choice(
            // DecimalInteger
            choice(
              "0",
              seq(
                // NonZeroDigit
                choice(
                  "1",
                  "2",
                  "3",
                  "4",
                  "5",
                  "6",
                  "7",
                  "8",
                  "9",
                ),
                optional(
                  // DecimalDigitsUS
                  repeat1(
                    // DecimalDigitUS
                    choice(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                      "_",
                    ),
                  ),
                ),
              ),
            ),
            // BinaryInteger
            seq(
              // BinPrefix
              choice(
                "0b",
                "0B",
              ),
              // BinaryDigitsNoSingleUS
              seq(
                optional(
                  // BinaryDigitsUS
                  repeat1(
                    // BinaryDigitUS
                    choice(
                      // BinaryDigit
                      choice(
                        "0",
                        "1",
                      ),
                      "_",
                    ),
                  ),
                ),
                // BinaryDigit
                choice(
                  "0",
                  "1",
                ),
                optional(
                  // BinaryDigitsUS
                  repeat1(
                    // BinaryDigitUS
                    choice(
                      // BinaryDigit
                      choice(
                        "0",
                        "1",
                      ),
                      "_",
                    ),
                  ),
                ),
              ),
            ),
            // HexadecimalInteger
            seq(
              // HexPrefix
              choice(
                "0x",
                "0X",
              ),
              // HexDigitsNoSingleUS
              choice(
                seq(
                  // HexDigit
                  choice(
                    // DecimalDigit
                    choice(
                      "0",
                      // NonZeroDigit
                      choice(
                        "1",
                        "2",
                        "3",
                        "4",
                        "5",
                        "6",
                        "7",
                        "8",
                        "9",
                      ),
                    ),
                    // HexLetter
                    choice(
                      "a",
                      "b",
                      "c",
                      "d",
                      "e",
                      "f",
                      "A",
                      "B",
                      "C",
                      "D",
                      "E",
                      "F",
                    ),
                  ),
                  optional(
                    // HexDigitsUS
                    repeat1(
                      // HexDigitUS
                      choice(
                        // HexDigit
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          // HexLetter
                          choice(
                            "a",
                            "b",
                            "c",
                            "d",
                            "e",
                            "f",
                            "A",
                            "B",
                            "C",
                            "D",
                            "E",
                            "F",
                          ),
                        ),
                        "_",
                      ),
                    ),
                  ),
                ),
                seq(
                  // HexDigitsUS
                  repeat1(
                    // HexDigitUS
                    choice(
                      // HexDigit
                      choice(
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                        // HexLetter
                        choice(
                          "a",
                          "b",
                          "c",
                          "d",
                          "e",
                          "f",
                          "A",
                          "B",
                          "C",
                          "D",
                          "E",
                          "F",
                        ),
                      ),
                      "_",
                    ),
                  ),
                  // HexDigit
                  choice(
                    // DecimalDigit
                    choice(
                      "0",
                      // NonZeroDigit
                      choice(
                        "1",
                        "2",
                        "3",
                        "4",
                        "5",
                        "6",
                        "7",
                        "8",
                        "9",
                      ),
                    ),
                    // HexLetter
                    choice(
                      "a",
                      "b",
                      "c",
                      "d",
                      "e",
                      "f",
                      "A",
                      "B",
                      "C",
                      "D",
                      "E",
                      "F",
                    ),
                  ),
                ),
              ),
            ),
          ),
          optional(
            // IntegerSuffix
            choice(
              "L",
              "u",
              "U",
              "Lu",
              "LU",
              "uL",
              "UL",
            ),
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#FloatLiteral
    float_literal: $ =>
      // Allow tree-sitter to disambiguate UFCS calls by defining two
      // versions of this token, with or without a trailing period.
      choice(
        $._float_literal_no_trailing_period,
        seq(
          choice(
            $._float_literal_trailing_period,
            // Hack around tree-sitter not being able to deal with ambiguous lexing rules
            alias(
              $.integer_literal,
              '_actually_a_float_literal_without_its_trailing_period',
            ),
          ),
          ".",
        )
      ),

    _float_literal_no_trailing_period: $ =>
      token(
        // FloatLiteral
        choice(
          seq(
            // Float
            choice(
              // DecimalFloat
              choice(
                seq(
                  // LeadingDecimal
                  choice(
                    // DecimalInteger
                    choice(
                      "0",
                      seq(
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                        optional(
                          // DecimalDigitsUS
                          repeat1(
                            // DecimalDigitUS
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              "_",
                            ),
                          ),
                        ),
                      ),
                    ),
                    seq(
                      "0",
                      // DecimalDigitsNoSingleUS
                      choice(
                        seq(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          optional(
                            // DecimalDigitsUS
                            repeat1(
                              // DecimalDigitUS
                              choice(
                                // DecimalDigit
                                choice(
                                  "0",
                                  // NonZeroDigit
                                  choice(
                                    "1",
                                    "2",
                                    "3",
                                    "4",
                                    "5",
                                    "6",
                                    "7",
                                    "8",
                                    "9",
                                  ),
                                ),
                                "_",
                              ),
                            ),
                          ),
                        ),
                        seq(
                          // DecimalDigitsUS
                          repeat1(
                            // DecimalDigitUS
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              "_",
                            ),
                          ),
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                  ".",
                  // optional(
                    // DecimalDigits
                    repeat1(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                    ),
                  // ),
                ),
                seq(
                  choice(
                    seq(
                      // DecimalDigits
                      repeat1(
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                      ),
                      ".",
                      // DecimalDigitsNoStartingUS
                      seq(
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                        optional(
                          // DecimalDigitsUS
                          repeat1(
                            // DecimalDigitUS
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              "_",
                            ),
                          ),
                        ),
                      ),
                    ),
                    seq(
                      ".",
                      // DecimalInteger
                      choice(
                        "0",
                        seq(
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                          optional(
                            // DecimalDigitsUS
                            repeat1(
                              // DecimalDigitUS
                              choice(
                                // DecimalDigit
                                choice(
                                  "0",
                                  // NonZeroDigit
                                  choice(
                                    "1",
                                    "2",
                                    "3",
                                    "4",
                                    "5",
                                    "6",
                                    "7",
                                    "8",
                                    "9",
                                  ),
                                ),
                                "_",
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                    // LeadingDecimal
                    choice(
                      // DecimalInteger
                      choice(
                        "0",
                        seq(
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                          optional(
                            // DecimalDigitsUS
                            repeat1(
                              // DecimalDigitUS
                              choice(
                                // DecimalDigit
                                choice(
                                  "0",
                                  // NonZeroDigit
                                  choice(
                                    "1",
                                    "2",
                                    "3",
                                    "4",
                                    "5",
                                    "6",
                                    "7",
                                    "8",
                                    "9",
                                  ),
                                ),
                                "_",
                              ),
                            ),
                          ),
                        ),
                      ),
                      seq(
                        "0",
                        // DecimalDigitsNoSingleUS
                        choice(
                          seq(
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                            optional(
                              // DecimalDigitsUS
                              repeat1(
                                // DecimalDigitUS
                                choice(
                                  // DecimalDigit
                                  choice(
                                    "0",
                                    // NonZeroDigit
                                    choice(
                                      "1",
                                      "2",
                                      "3",
                                      "4",
                                      "5",
                                      "6",
                                      "7",
                                      "8",
                                      "9",
                                    ),
                                  ),
                                  "_",
                                ),
                              ),
                            ),
                          ),
                          seq(
                            // DecimalDigitsUS
                            repeat1(
                              // DecimalDigitUS
                              choice(
                                // DecimalDigit
                                choice(
                                  "0",
                                  // NonZeroDigit
                                  choice(
                                    "1",
                                    "2",
                                    "3",
                                    "4",
                                    "5",
                                    "6",
                                    "7",
                                    "8",
                                    "9",
                                  ),
                                ),
                                "_",
                              ),
                            ),
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                  // DecimalExponent
                  seq(
                    // DecimalExponentStart
                    choice(
                      "e",
                      "E",
                      "e+",
                      "E+",
                      "e-",
                      "E-",
                    ),
                    // DecimalDigitsNoSingleUS
                    choice(
                      seq(
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                        optional(
                          // DecimalDigitsUS
                          repeat1(
                            // DecimalDigitUS
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              "_",
                            ),
                          ),
                        ),
                      ),
                      seq(
                        // DecimalDigitsUS
                        repeat1(
                          // DecimalDigitUS
                          choice(
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                            "_",
                          ),
                        ),
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
                seq(
                  ".",
                  // DecimalInteger
                  choice(
                    "0",
                    seq(
                      // NonZeroDigit
                      choice(
                        "1",
                        "2",
                        "3",
                        "4",
                        "5",
                        "6",
                        "7",
                        "8",
                        "9",
                      ),
                      optional(
                        // DecimalDigitsUS
                        repeat1(
                          // DecimalDigitUS
                          choice(
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                            "_",
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
              // HexFloat
              seq(
                // HexPrefix
                choice(
                  "0x",
                  "0X",
                ),
                choice(
                  seq(
                    optional(
                      // HexDigitsNoSingleUS
                      choice(
                        seq(
                          // HexDigit
                          choice(
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                            // HexLetter
                            choice(
                              "a",
                              "b",
                              "c",
                              "d",
                              "e",
                              "f",
                              "A",
                              "B",
                              "C",
                              "D",
                              "E",
                              "F",
                            ),
                          ),
                          optional(
                            // HexDigitsUS
                            repeat1(
                              // HexDigitUS
                              choice(
                                // HexDigit
                                choice(
                                  // DecimalDigit
                                  choice(
                                    "0",
                                    // NonZeroDigit
                                    choice(
                                      "1",
                                      "2",
                                      "3",
                                      "4",
                                      "5",
                                      "6",
                                      "7",
                                      "8",
                                      "9",
                                    ),
                                  ),
                                  // HexLetter
                                  choice(
                                    "a",
                                    "b",
                                    "c",
                                    "d",
                                    "e",
                                    "f",
                                    "A",
                                    "B",
                                    "C",
                                    "D",
                                    "E",
                                    "F",
                                  ),
                                ),
                                "_",
                              ),
                            ),
                          ),
                        ),
                        seq(
                          // HexDigitsUS
                          repeat1(
                            // HexDigitUS
                            choice(
                              // HexDigit
                              choice(
                                // DecimalDigit
                                choice(
                                  "0",
                                  // NonZeroDigit
                                  choice(
                                    "1",
                                    "2",
                                    "3",
                                    "4",
                                    "5",
                                    "6",
                                    "7",
                                    "8",
                                    "9",
                                  ),
                                ),
                                // HexLetter
                                choice(
                                  "a",
                                  "b",
                                  "c",
                                  "d",
                                  "e",
                                  "f",
                                  "A",
                                  "B",
                                  "C",
                                  "D",
                                  "E",
                                  "F",
                                ),
                              ),
                              "_",
                            ),
                          ),
                          // HexDigit
                          choice(
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                            // HexLetter
                            choice(
                              "a",
                              "b",
                              "c",
                              "d",
                              "e",
                              "f",
                              "A",
                              "B",
                              "C",
                              "D",
                              "E",
                              "F",
                            ),
                          ),
                        ),
                      ),
                    ),
                    ".",
                    // HexDigitsNoStartingUS
                    seq(
                      // HexDigit
                      choice(
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                        // HexLetter
                        choice(
                          "a",
                          "b",
                          "c",
                          "d",
                          "e",
                          "f",
                          "A",
                          "B",
                          "C",
                          "D",
                          "E",
                          "F",
                        ),
                      ),
                      optional(
                        // HexDigitsUS
                        repeat1(
                          // HexDigitUS
                          choice(
                            // HexDigit
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              // HexLetter
                              choice(
                                "a",
                                "b",
                                "c",
                                "d",
                                "e",
                                "f",
                                "A",
                                "B",
                                "C",
                                "D",
                                "E",
                                "F",
                              ),
                            ),
                            "_",
                          ),
                        ),
                      ),
                    ),
                  ),
                  // HexDigitsNoSingleUS
                  choice(
                    seq(
                      // HexDigit
                      choice(
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                        // HexLetter
                        choice(
                          "a",
                          "b",
                          "c",
                          "d",
                          "e",
                          "f",
                          "A",
                          "B",
                          "C",
                          "D",
                          "E",
                          "F",
                        ),
                      ),
                      optional(
                        // HexDigitsUS
                        repeat1(
                          // HexDigitUS
                          choice(
                            // HexDigit
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              // HexLetter
                              choice(
                                "a",
                                "b",
                                "c",
                                "d",
                                "e",
                                "f",
                                "A",
                                "B",
                                "C",
                                "D",
                                "E",
                                "F",
                              ),
                            ),
                            "_",
                          ),
                        ),
                      ),
                    ),
                    seq(
                      // HexDigitsUS
                      repeat1(
                        // HexDigitUS
                        choice(
                          // HexDigit
                          choice(
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                            // HexLetter
                            choice(
                              "a",
                              "b",
                              "c",
                              "d",
                              "e",
                              "f",
                              "A",
                              "B",
                              "C",
                              "D",
                              "E",
                              "F",
                            ),
                          ),
                          "_",
                        ),
                      ),
                      // HexDigit
                      choice(
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                        // HexLetter
                        choice(
                          "a",
                          "b",
                          "c",
                          "d",
                          "e",
                          "f",
                          "A",
                          "B",
                          "C",
                          "D",
                          "E",
                          "F",
                        ),
                      ),
                    ),
                  ),
                ),
                // HexExponent
                seq(
                  // HexExponentStart
                  choice(
                    "p",
                    "P",
                    "p+",
                    "P+",
                    "p-",
                    "P-",
                  ),
                  // DecimalDigitsNoSingleUS
                  choice(
                    seq(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                      optional(
                        // DecimalDigitsUS
                        repeat1(
                          // DecimalDigitUS
                          choice(
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                            "_",
                          ),
                        ),
                      ),
                    ),
                    seq(
                      // DecimalDigitsUS
                      repeat1(
                        // DecimalDigitUS
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          "_",
                        ),
                      ),
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
            optional(
              // Suffix
              choice(
                seq(
                  choice(
                    // FloatSuffix
                    choice(
                      "f",
                      "F",
                    ),
                    // RealSuffix
                    "L",
                  ),
                  optional(
                    // ImaginarySuffix
                    "i",
                  ),
                ),
                // ImaginarySuffix
                "i",
              ),
            ),
          ),
          seq(
            // Integer
            choice(
              // DecimalInteger
              choice(
                "0",
                seq(
                  // NonZeroDigit
                  choice(
                    "1",
                    "2",
                    "3",
                    "4",
                    "5",
                    "6",
                    "7",
                    "8",
                    "9",
                  ),
                  optional(
                    // DecimalDigitsUS
                    repeat1(
                      // DecimalDigitUS
                      choice(
                        // DecimalDigit
                        choice(
                          "0",
                          // NonZeroDigit
                          choice(
                            "1",
                            "2",
                            "3",
                            "4",
                            "5",
                            "6",
                            "7",
                            "8",
                            "9",
                          ),
                        ),
                        "_",
                      ),
                    ),
                  ),
                ),
              ),
              // BinaryInteger
              seq(
                // BinPrefix
                choice(
                  "0b",
                  "0B",
                ),
                // BinaryDigitsNoSingleUS
                seq(
                  optional(
                    // BinaryDigitsUS
                    repeat1(
                      // BinaryDigitUS
                      choice(
                        // BinaryDigit
                        choice(
                          "0",
                          "1",
                        ),
                        "_",
                      ),
                    ),
                  ),
                  // BinaryDigit
                  choice(
                    "0",
                    "1",
                  ),
                  optional(
                    // BinaryDigitsUS
                    repeat1(
                      // BinaryDigitUS
                      choice(
                        // BinaryDigit
                        choice(
                          "0",
                          "1",
                        ),
                        "_",
                      ),
                    ),
                  ),
                ),
              ),
              // HexadecimalInteger
              seq(
                // HexPrefix
                choice(
                  "0x",
                  "0X",
                ),
                // HexDigitsNoSingleUS
                choice(
                  seq(
                    // HexDigit
                    choice(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                      // HexLetter
                      choice(
                        "a",
                        "b",
                        "c",
                        "d",
                        "e",
                        "f",
                        "A",
                        "B",
                        "C",
                        "D",
                        "E",
                        "F",
                      ),
                    ),
                    optional(
                      // HexDigitsUS
                      repeat1(
                        // HexDigitUS
                        choice(
                          // HexDigit
                          choice(
                            // DecimalDigit
                            choice(
                              "0",
                              // NonZeroDigit
                              choice(
                                "1",
                                "2",
                                "3",
                                "4",
                                "5",
                                "6",
                                "7",
                                "8",
                                "9",
                              ),
                            ),
                            // HexLetter
                            choice(
                              "a",
                              "b",
                              "c",
                              "d",
                              "e",
                              "f",
                              "A",
                              "B",
                              "C",
                              "D",
                              "E",
                              "F",
                            ),
                          ),
                          "_",
                        ),
                      ),
                    ),
                  ),
                  seq(
                    // HexDigitsUS
                    repeat1(
                      // HexDigitUS
                      choice(
                        // HexDigit
                        choice(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          // HexLetter
                          choice(
                            "a",
                            "b",
                            "c",
                            "d",
                            "e",
                            "f",
                            "A",
                            "B",
                            "C",
                            "D",
                            "E",
                            "F",
                          ),
                        ),
                        "_",
                      ),
                    ),
                    // HexDigit
                    choice(
                      // DecimalDigit
                      choice(
                        "0",
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                      ),
                      // HexLetter
                      choice(
                        "a",
                        "b",
                        "c",
                        "d",
                        "e",
                        "f",
                        "A",
                        "B",
                        "C",
                        "D",
                        "E",
                        "F",
                      ),
                    ),
                  ),
                ),
              ),
            ),
            choice(
              // FloatSuffix
              choice(
                "f",
                "F",
              ),
              seq(
                optional(
                  choice(
                    // FloatSuffix
                    choice(
                      "f",
                      "F",
                    ),
                    // RealSuffix
                    "L",
                  ),
                ),
                // ImaginarySuffix
                "i",
              ),
            ),
          ),
        ),
      ),

    _float_literal_trailing_period: $ =>
      token(
        // FloatLiteral
        choice(
          seq(
            // Float
            choice(
              // DecimalFloat
              choice(
                seq(
                  // LeadingDecimal
                  choice(
                    // DecimalInteger
                    choice(
                      "0",
                      seq(
                        // NonZeroDigit
                        choice(
                          "1",
                          "2",
                          "3",
                          "4",
                          "5",
                          "6",
                          "7",
                          "8",
                          "9",
                        ),
                        optional(
                          // DecimalDigitsUS
                          repeat1(
                            // DecimalDigitUS
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              "_",
                            ),
                          ),
                        ),
                      ),
                    ),
                    seq(
                      "0",
                      // DecimalDigitsNoSingleUS
                      choice(
                        seq(
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                          optional(
                            // DecimalDigitsUS
                            repeat1(
                              // DecimalDigitUS
                              choice(
                                // DecimalDigit
                                choice(
                                  "0",
                                  // NonZeroDigit
                                  choice(
                                    "1",
                                    "2",
                                    "3",
                                    "4",
                                    "5",
                                    "6",
                                    "7",
                                    "8",
                                    "9",
                                  ),
                                ),
                                "_",
                              ),
                            ),
                          ),
                        ),
                        seq(
                          // DecimalDigitsUS
                          repeat1(
                            // DecimalDigitUS
                            choice(
                              // DecimalDigit
                              choice(
                                "0",
                                // NonZeroDigit
                                choice(
                                  "1",
                                  "2",
                                  "3",
                                  "4",
                                  "5",
                                  "6",
                                  "7",
                                  "8",
                                  "9",
                                ),
                              ),
                              "_",
                            ),
                          ),
                          // DecimalDigit
                          choice(
                            "0",
                            // NonZeroDigit
                            choice(
                              "1",
                              "2",
                              "3",
                              "4",
                              "5",
                              "6",
                              "7",
                              "8",
                              "9",
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                  // ".",
                ),
              ),
            ),
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/lex.html#Keyword
    keyword: $ =>
      choice(
        "abstract",
        "alias",
        "align",
        "asm",
        "assert",
        "auto",
        "body",
        "bool",
        "break",
        "byte",
        "case",
        "cast",
        "catch",
        "cdouble",
        "cent",
        "cfloat",
        "char",
        "class",
        "const",
        "continue",
        "creal",
        "dchar",
        "debug",
        "default",
        "delegate",
        "delete",
        "deprecated",
        "do",
        "double",
        "else",
        "enum",
        "export",
        "extern",
        "false",
        "final",
        "finally",
        "float",
        "for",
        "foreach",
        "foreach_reverse",
        "function",
        "goto",
        "idouble",
        "if",
        "ifloat",
        "immutable",
        "import",
        "in",
        "inout",
        "int",
        "interface",
        "invariant",
        "ireal",
        "is",
        "lazy",
        "long",
        "macro",
        "mixin",
        "module",
        "new",
        "nothrow",
        "null",
        "out",
        "override",
        "package",
        "pragma",
        "private",
        "protected",
        "public",
        "pure",
        "real",
        "ref",
        "return",
        "scope",
        "shared",
        "short",
        "static",
        "struct",
        "super",
        "switch",
        "synchronized",
        "template",
        "this",
        "throw",
        "true",
        "try",
        "typeid",
        "typeof",
        "ubyte",
        "ucent",
        "uint",
        "ulong",
        "union",
        "unittest",
        "ushort",
        "version",
        "void",
        "wchar",
        "while",
        "with",
        "__FILE__",
        "__FILE_FULL_PATH__",
        "__MODULE__",
        "__LINE__",
        "__FUNCTION__",
        "__PRETTY_FUNCTION__",
        "__gshared",
        "__traits",
        "__vector",
        "__parameters",
      ),

    // ---

    // https://dlang.org/spec/lex.html#SpecialTokenSequence
    special_token_sequence: $ =>
      seq(
        "#",
        "line",
        choice(
          $.integer_literal,
          "__LINE__",
        ),
        optional(
          $.filespec,
        ),
        $._end_of_line,
      ),

    // ---

    // https://dlang.org/spec/lex.html#Filespec
    filespec: $ =>
      token(
        // Filespec
        seq(
          "\"",
          optional(
            // Characters
            repeat1(
              // Character
              /[^"]/,
            ),
          ),
          "\"",
        ),
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/module.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/module.html#Module
    module: $ =>
      choice(
        seq(
          $.module_declaration,
          optional(
            $.decl_defs,
          ),
        ),
        $.decl_defs,
      ),

    // https://dlang.org/spec/module.html#DeclDefs
    decl_defs: $ =>
      repeat1(
        $._decl_def,
      ),

    // https://dlang.org/spec/module.html#DeclDef
    _decl_def: $ =>
      choice(
        $.attribute_specifier,
        $._declaration,
        $._maybe_constructor,
        $.destructor,
        $.postblit,
        $.allocator,
        $.deallocator,
        $.invariant,
        $.unit_test,
        $.alias_this,
        $.static_constructor,
        $.static_destructor,
        $.shared_static_constructor,
        $.shared_static_destructor,
        $.conditional_declaration,
        $.debug_specification,
        $.version_specification,
        $.static_assert,
        $.template_mixin_declaration,
        $.template_mixin,
        $.mixin_declaration,
        $.empty_declaration,
      ),

    // https://dlang.org/spec/module.html#EmptyDeclaration
    empty_declaration: $ =>
      ";",

    // ---

    // https://dlang.org/spec/module.html#ModuleDeclaration
    module_declaration: $ =>
      seq(
        optional(
          $.module_attributes,
        ),
        "module",
        $._maybe_module_fully_qualified_name,
        ";",
      ),

    // https://dlang.org/spec/module.html#ModuleAttributes
    module_attributes: $ =>
      repeat1(
        $._module_attribute,
      ),

    // https://dlang.org/spec/module.html#ModuleAttribute
    _module_attribute: $ =>
      choice(
        $.deprecated_attribute,
        $.user_defined_attribute,
      ),

    // https://dlang.org/spec/module.html#ModuleFullyQualifiedName
    _maybe_module_fully_qualified_name: $ =>
      choice(
        $.module_name,
        $.module_fully_qualified_name,
      ),

    module_fully_qualified_name: $ =>
      seq(
        $.packages,
        ".",
        $.module_name,
      ),

    // https://dlang.org/spec/module.html#ModuleName
    module_name: $ =>
      $.identifier,

    // https://dlang.org/spec/module.html#Packages
    packages: $ =>
      seq(
        repeat(
          seq(
            $.package_name,
            ".",
          ),
        ),
        $.package_name,
      ),

    // https://dlang.org/spec/module.html#PackageName
    package_name: $ =>
      $.identifier,

    // ---

    // https://dlang.org/spec/module.html#ImportDeclaration
    import_declaration: $ =>
      seq(
        optional(
          "static",
        ),
        "import",
        $.import_list,
        ";",
      ),

    // https://dlang.org/spec/module.html#ImportList
    import_list: $ =>
      seq(
        repeat(
          seq(
            $.import,
            ",",
          ),
        ),
        choice(
          $.import,
          $.import_bindings,
        ),
      ),

    // https://dlang.org/spec/module.html#Import
    import: $ =>
      seq(
        optional(
          seq(
            $.module_alias_identifier,
            "=",
          ),
        ),
        $._maybe_module_fully_qualified_name,
      ),

    // https://dlang.org/spec/module.html#ImportBindings
    import_bindings: $ =>
      seq(
        $.import,
        ":",
        $.import_bind_list,
      ),

    // https://dlang.org/spec/module.html#ImportBindList
    import_bind_list: $ =>
      seq(
        repeat(
          seq(
            $.import_bind,
            ",",
          ),
        ),
        $.import_bind,
      ),

    // https://dlang.org/spec/module.html#ImportBind
    import_bind: $ =>
      seq(
        $.identifier,
        optional(
          seq(
            "=",
            $.identifier,
          ),
        ),
      ),

    // https://dlang.org/spec/module.html#ModuleAliasIdentifier
    module_alias_identifier: $ =>
      $.identifier,

    // ---

    // https://dlang.org/spec/module.html#MixinDeclaration
    mixin_declaration: $ =>
      seq(
        "mixin",
        "(",
        $.argument_list,
        ")",
        ";",
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/declaration.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/declaration.html#Declaration
    _declaration: $ =>
      choice(
        $._maybe_func_declaration,
        $._maybe_var_declarations,
        $.alias_declaration,
        $.alias_assign,
        $._aggregate_declaration,
        $._maybe_enum_declaration,
        $.template_declaration,
        $.import_declaration,
        $.conditional_declaration,
        $.static_foreach_declaration,
        $.static_assert,
      ),

    // ---

    // https://dlang.org/spec/declaration.html#VarDeclarations
    _maybe_var_declarations: $ =>
      choice(
        $.auto_declaration,
        $.var_declarations,
      ),

    var_declarations: $ =>
      seq(
        optional(
          $.storage_classes,
        ),
        $._maybe_basic_type,
        $.declarators,
        ";",
      ),

    // https://dlang.org/spec/declaration.html#Declarators
    declarators: $ =>
      seq(
        $._maybe_declarator_initializer,
        optional(
          seq(
            ",",
            $.declarator_identifier_list,
          ),
        ),
      ),

    // https://dlang.org/spec/declaration.html#DeclaratorInitializer
    _maybe_declarator_initializer: $ =>
      choice(
        $.var_declarator,
        $.alt_declarator,
        $.declarator_initializer,
      ),

    declarator_initializer: $ =>
      seq(
        choice(
          seq(
            $.var_declarator,
            optional(
              $.template_parameters,
            ),
          ),
          $.alt_declarator,
        ),
        "=",
        $._initializer,
      ),

    // https://dlang.org/spec/declaration.html#DeclaratorIdentifierList
    declarator_identifier_list: $ =>
      seq(
        repeat(
          seq(
            $._declarator_identifier,
            ",",
          ),
        ),
        $._declarator_identifier,
      ),

    // https://dlang.org/spec/declaration.html#DeclaratorIdentifier
    _declarator_identifier: $ =>
      choice(
        $.var_declarator_identifier,
        $.alt_declarator_identifier,
      ),

    // https://dlang.org/spec/declaration.html#VarDeclaratorIdentifier
    var_declarator_identifier: $ =>
      seq(
        $.identifier,
        optional(
          seq(
            optional(
              $.template_parameters,
            ),
            "=",
            $._initializer,
          ),
        ),
      ),

    // https://dlang.org/spec/declaration.html#AltDeclaratorIdentifier
    alt_declarator_identifier: $ =>
      seq(
        optional(
          $.type_suffixes,
        ),
        $.identifier,
        optional(
          $.alt_declarator_suffixes,
        ),
        optional(
          seq(
            "=",
            $._initializer,
          ),
        ),
      ),

    // https://dlang.org/spec/declaration.html#Declarator
    _declarator: $ =>
      choice(
        $.var_declarator,
        $.alt_declarator,
      ),

    // https://dlang.org/spec/declaration.html#VarDeclarator
    var_declarator: $ =>
      seq(
        optional(
          $.type_suffixes,
        ),
        $.identifier,
      ),

    // https://dlang.org/spec/declaration.html#AltDeclarator
    alt_declarator: $ =>
      seq(
        optional(
          $.type_suffixes,
        ),
        choice(
          seq(
            $.identifier,
            $.alt_declarator_suffixes,
          ),
          seq(
            "(",
            $.alt_declarator_inner,
            ")",
            optional(
              choice(
                $.alt_func_declarator_suffix,
                $.alt_declarator_suffixes,
              ),
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/declaration.html#AltDeclaratorInner
    alt_declarator_inner: $ =>
      choice(
        seq(
          optional(
            $.type_suffixes,
          ),
          $.identifier,
          optional(
            $.alt_func_declarator_suffix,
          ),
        ),
        $.alt_declarator,
      ),

    // https://dlang.org/spec/declaration.html#AltDeclaratorSuffixes
    alt_declarator_suffixes: $ =>
      repeat1(
        $.alt_declarator_suffix,
      ),

    // https://dlang.org/spec/declaration.html#AltDeclaratorSuffix
    alt_declarator_suffix: $ =>
      seq(
        "[",
        optional(
          choice(
            $._maybe_assign_expression,
            $.type,
          ),
        ),
        "]",
      ),

    // https://dlang.org/spec/declaration.html#AltFuncDeclaratorSuffix
    alt_func_declarator_suffix: $ =>
      seq(
        $.parameters,
        optional(
          $.member_function_attributes,
        ),
      ),

    // ---

    // https://dlang.org/spec/declaration.html#StorageClasses
    storage_classes: $ =>
      repeat1(
        $._maybe_storage_class,
      ),

    // https://dlang.org/spec/declaration.html#StorageClass
    _maybe_storage_class: $ =>
      choice(
        $.linkage_attribute,
        $.align_attribute,
        $._maybe_at_attribute,
        $.property,
        $.storage_class,
      ),

    storage_class: $ =>
      choice(
        "deprecated",
        "enum",
        "static",
        "extern",
        "abstract",
        "final",
        "override",
        "synchronized",
        "auto",
        "scope",
        "const",
        "immutable",
        "inout",
        "shared",
        "__gshared",
        "nothrow",
        "pure",
        "ref",
      ),

    // ---

    // https://dlang.org/spec/declaration.html#Initializer
    _initializer: $ =>
      choice(
        $.void_initializer,
        $._non_void_initializer,
      ),

    // https://dlang.org/spec/declaration.html#NonVoidInitializer
    _non_void_initializer: $ =>
      choice(
        $.exp_initializer,
        $.array_initializer,
        $.struct_initializer,
      ),

    // https://dlang.org/spec/declaration.html#ExpInitializer
    exp_initializer: $ =>
      $._maybe_assign_expression,

    // https://dlang.org/spec/declaration.html#ArrayInitializer
    array_initializer: $ =>
      seq(
        "[",
        optional(
          $.array_member_initializations,
        ),
        "]",
      ),

    // https://dlang.org/spec/declaration.html#ArrayMemberInitializations
    array_member_initializations: $ =>
      seq(
        repeat(
          seq(
            $.array_member_initialization,
            ",",
          ),
        ),
        $.array_member_initialization,
        optional(
          ",",
        ),
      ),

    // https://dlang.org/spec/declaration.html#ArrayMemberInitialization
    array_member_initialization: $ =>
      seq(
        optional(
          seq(
            $._maybe_assign_expression,
            ":",
          ),
        ),
        $._non_void_initializer,
      ),

    // https://dlang.org/spec/declaration.html#StructInitializer
    struct_initializer: $ =>
      seq(
        "{",
        optional(
          $.struct_member_initializers,
        ),
        "}",
      ),

    // https://dlang.org/spec/declaration.html#StructMemberInitializers
    struct_member_initializers: $ =>
      seq(
        repeat(
          seq(
            $.struct_member_initializer,
            ",",
          ),
        ),
        $.struct_member_initializer,
        optional(
          ",",
        ),
      ),

    // https://dlang.org/spec/declaration.html#StructMemberInitializer
    struct_member_initializer: $ =>
      seq(
        optional(
          seq(
            $.identifier,
            ":",
          ),
        ),
        $._non_void_initializer,
      ),

    // ---

    // https://dlang.org/spec/declaration.html#AutoDeclaration
    auto_declaration: $ =>
      seq(
        $.storage_classes,
        $.auto_assignments,
        ";",
      ),

    // https://dlang.org/spec/declaration.html#AutoAssignments
    auto_assignments: $ =>
      seq(
        repeat(
          seq(
            $.auto_assignment,
            ",",
          ),
        ),
        $.auto_assignment,
      ),

    // https://dlang.org/spec/declaration.html#AutoAssignment
    auto_assignment: $ =>
      seq(
        $.identifier,
        optional(
          $.template_parameters,
        ),
        "=",
        $._initializer,
      ),

    // ---

    // https://dlang.org/spec/declaration.html#AliasDeclaration
    alias_declaration: $ =>
      seq(
        "alias",
        choice(
          seq(
            optional(
              $.storage_classes,
            ),
            $._maybe_basic_type,
            choice(
              $.declarators,
              $.func_declarator,
            ),
          ),
          $.alias_assignments,
        ),
        ";",
      ),

    // https://dlang.org/spec/declaration.html#AliasAssignments
    alias_assignments: $ =>
      seq(
        repeat(
          seq(
            $.alias_assignment,
            ",",
          ),
        ),
        $.alias_assignment,
      ),

    // https://dlang.org/spec/declaration.html#AliasAssignment
    alias_assignment: $ =>
      seq(
        $.identifier,
        optional(
          $.template_parameters,
        ),
        "=",
        choice(
          seq(
            optional(
              $.storage_classes,
            ),
            choice(
              $.type,
              seq(
                $._maybe_basic_type,
                $.parameters,
                optional(
                  $.member_function_attributes,
                ),
              ),
            ),
          ),
          $._maybe_function_literal,
        ),
      ),

    // ---

    // https://dlang.org/spec/declaration.html#AliasAssign
    alias_assign: $ =>
      seq(
        $.identifier,
        "=",
        $.type,
      ),

    // ---

    // https://dlang.org/spec/declaration.html#VoidInitializer
    void_initializer: $ =>
      "void",

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/type.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/type.html#Type
    type: $ =>
      seq(
        optional(
          $.type_ctors,
        ),
        $._maybe_basic_type,
        optional(
          $.type_suffixes,
        ),
      ),

    // https://dlang.org/spec/type.html#TypeCtors
    type_ctors: $ =>
      repeat1(
        $.type_ctor,
      ),

    // https://dlang.org/spec/type.html#TypeCtor
    type_ctor: $ =>
      choice(
        "const",
        "immutable",
        "inout",
        "shared",
      ),

    // https://dlang.org/spec/type.html#BasicType
    _maybe_basic_type: $ =>
      choice(
        $.fundamental_type,
        $.qualified_identifier,
        $.typeof,
        $.vector,
        $.traits_expression,
        $.mixin_type,
        $.basic_type,
      ),

    basic_type: $ =>
      choice(
        seq(
          optional(
            $.typeof,
          ),
          ".",
          $.qualified_identifier,
        ),
        seq(
          $.type_ctor,
          "(",
          $.type,
          ")",
        ),
      ),

    // https://dlang.org/spec/type.html#Vector
    vector: $ =>
      seq(
        "__vector",
        "(",
        $.vector_base_type,
        ")",
      ),

    // https://dlang.org/spec/type.html#VectorBaseType
    vector_base_type: $ =>
      $.type,

    // https://dlang.org/spec/type.html#FundamentalType
    fundamental_type: $ =>
      choice(
        "bool",
        "byte",
        "ubyte",
        "short",
        "ushort",
        "int",
        "uint",
        "long",
        "ulong",
        "cent",
        "ucent",
        "char",
        "wchar",
        "dchar",
        "float",
        "double",
        "real",
        "ifloat",
        "idouble",
        "ireal",
        "cfloat",
        "cdouble",
        "creal",
        "void",
      ),

    // https://dlang.org/spec/type.html#TypeSuffixes
    type_suffixes: $ =>
      repeat1(
        $.type_suffix,
      ),

    // https://dlang.org/spec/type.html#TypeSuffix
    type_suffix: $ =>
      choice(
        "*",
        seq(
          "[",
          optional(
            choice(
              seq(
                $._maybe_assign_expression,
                optional(
                  seq(
                    "..",
                    $._maybe_assign_expression,
                  ),
                ),
              ),
              $.type,
            ),
          ),
          "]",
        ),
        seq(
          "delegate",
          $.parameters,
          optional(
            $.member_function_attributes,
          ),
        ),
        seq(
          "function",
          $.parameters,
          optional(
            $.function_attributes,
          ),
        ),
      ),

    // https://dlang.org/spec/type.html#QualifiedIdentifier
    qualified_identifier: $ =>
      seq(
        choice(
          seq(
            $.identifier,
            optional(
              seq(
                "[",
                $._maybe_assign_expression,
                "]",
              ),
            ),
          ),
          $.template_instance,
        ),
        optional(
          seq(
            ".",
            $.qualified_identifier,
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/type.html#Typeof
    typeof: $ =>
      seq(
        "typeof",
        "(",
        choice(
          $.expression,
          "return",
        ),
        ")",
      ),

    // ---

    // https://dlang.org/spec/type.html#MixinType
    mixin_type: $ =>
      seq(
        "mixin",
        "(",
        $.argument_list,
        ")",
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/attribute.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/attribute.html#AttributeSpecifier
    attribute_specifier: $ =>
      seq(
        $._maybe_attribute,
        choice(
          ":",
          $._maybe_declaration_block,
        ),
      ),

    // https://dlang.org/spec/attribute.html#Attribute
    _maybe_attribute: $ =>
      choice(
        $.linkage_attribute,
        $.align_attribute,
        $.deprecated_attribute,
        $.visibility_attribute,
        $.pragma,
        $._maybe_at_attribute,
        $.function_attribute_kwd,
        $.attribute,
      ),

    attribute: $ =>
      choice(
        "static",
        "extern",
        "abstract",
        "final",
        "override",
        "synchronized",
        "auto",
        "scope",
        "const",
        "immutable",
        "inout",
        "shared",
        "__gshared",
        "ref",
        "return",
      ),

    // https://dlang.org/spec/attribute.html#FunctionAttributeKwd
    function_attribute_kwd: $ =>
      choice(
        "nothrow",
        "pure",
      ),

    // https://dlang.org/spec/attribute.html#AtAttribute
    _maybe_at_attribute: $ =>
      choice(
        $.property,
        $.user_defined_attribute,
        $.at_attribute,
      ),

    at_attribute: $ =>
      seq(
        "@",
        choice(
          "disable",
          "nogc",
          "live",
          "safe",
          "system",
          "trusted",
        ),
      ),

    // https://dlang.org/spec/attribute.html#Property
    property: $ =>
      seq(
        "@",
        "property",
      ),

    // https://dlang.org/spec/attribute.html#DeclarationBlock
    _maybe_declaration_block: $ =>
      choice(
        $._decl_def,
        $.declaration_block,
      ),

    declaration_block: $ =>
      seq(
        "{",
        optional(
          $.decl_defs,
        ),
        "}",
      ),

    // ---

    // https://dlang.org/spec/attribute.html#LinkageAttribute
    linkage_attribute: $ =>
      seq(
        "extern",
        "(",
        choice(
          $.linkage_type,
          seq(
            "C++",
            ",",
            optional(
              choice(
                $.qualified_identifier,
                $.namespace_list,
              ),
            ),
          ),
        ),
        ")",
      ),

    // https://dlang.org/spec/attribute.html#LinkageType
    linkage_type: $ =>
      choice(
        "C",
        "C++",
        "D",
        "Windows",
        "System",
        "Objective-C",
      ),

    // https://dlang.org/spec/attribute.html#NamespaceList
    namespace_list: $ =>
      seq(
        repeat(
          seq(
            $._maybe_conditional_expression,
            ",",
          ),
        ),
        $._maybe_conditional_expression,
        optional(
          ",",
        ),
      ),

    // ---

    // https://dlang.org/spec/attribute.html#AlignAttribute
    align_attribute: $ =>
      seq(
        "align",
        optional(
          seq(
            "(",
            $._maybe_assign_expression,
            ")",
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/attribute.html#DeprecatedAttribute
    deprecated_attribute: $ =>
      seq(
        "deprecated",
        optional(
          seq(
            "(",
            $._maybe_assign_expression,
            ")",
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/attribute.html#VisibilityAttribute
    visibility_attribute: $ =>
      choice(
        "private",
        seq(
          "package",
          optional(
            seq(
              "(",
              $.qualified_identifier,
              ")",
            ),
          ),
        ),
        "protected",
        "public",
        "export",
      ),

    // ---

    // https://dlang.org/spec/attribute.html#UserDefinedAttribute
    user_defined_attribute: $ =>
      seq(
        "@",
        choice(
          seq(
            "(",
            $.argument_list,
            ")",
          ),
          seq(
            choice(
              $.identifier,
              $.template_instance,
            ),
            optional(
              seq(
                "(",
                optional(
                  $.argument_list,
                ),
                ")",
              ),
            ),
          ),
        ),
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/pragma.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/pragma.html#PragmaStatement
    pragma_statement: $ =>
      seq(
        $.pragma,
        choice(
          ";",
          $._no_scope_statement,
        ),
      ),

    // https://dlang.org/spec/pragma.html#Pragma
    pragma: $ =>
      seq(
        "pragma",
        "(",
        $.identifier,
        optional(
          seq(
            ",",
            $.argument_list,
          ),
        ),
        ")",
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/expression.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/expression.html#Expression
    expression: $ =>
      $._maybe_comma_expression,

    // https://dlang.org/spec/expression.html#CommaExpression
    _maybe_comma_expression: $ =>
      choice(
        $._maybe_assign_expression,
        $.comma_expression,
      ),

    comma_expression: $ =>
      seq(
        $._maybe_comma_expression,
        ",",
        $._maybe_assign_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#AssignExpression
    _maybe_assign_expression: $ =>
      choice(
        $._maybe_conditional_expression,
        $.assign_expression,
      ),

    assign_expression: $ =>
      seq(
        $._maybe_conditional_expression,
        choice(
          "=",
          "+=",
          "-=",
          "*=",
          "/=",
          "%=",
          "&=",
          "|=",
          "^=",
          "~=",
          "<<=",
          ">>=",
          ">>>=",
          "^^=",
        ),
        $._maybe_assign_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#ConditionalExpression
    _maybe_conditional_expression: $ =>
      choice(
        $._maybe_or_or_expression,
        $.conditional_expression,
      ),

    conditional_expression: $ =>
      seq(
        $._maybe_or_or_expression,
        "?",
        $.expression,
        ":",
        $._maybe_conditional_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#OrOrExpression
    _maybe_or_or_expression: $ =>
      choice(
        $._maybe_and_and_expression,
        $.or_or_expression,
      ),

    or_or_expression: $ =>
      seq(
        $._maybe_or_or_expression,
        "||",
        $._maybe_and_and_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#AndAndExpression
    _maybe_and_and_expression: $ =>
      choice(
        $._maybe_or_expression,
        $.and_and_expression,
      ),

    and_and_expression: $ =>
      seq(
        $._maybe_and_and_expression,
        "&&",
        $._maybe_or_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#OrExpression
    _maybe_or_expression: $ =>
      choice(
        $._maybe_xor_expression,
        $.or_expression,
      ),

    or_expression: $ =>
      seq(
        $._maybe_or_expression,
        "|",
        $._maybe_xor_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#XorExpression
    _maybe_xor_expression: $ =>
      choice(
        $._maybe_and_expression,
        $.xor_expression,
      ),

    xor_expression: $ =>
      seq(
        $._maybe_xor_expression,
        "^",
        $._maybe_and_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#AndExpression
    _maybe_and_expression: $ =>
      choice(
        $._cmp_expression,
        $.and_expression,
      ),

    and_expression: $ =>
      seq(
        $._maybe_and_expression,
        "&",
        $._cmp_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#CmpExpression
    _cmp_expression: $ =>
      choice(
        $._maybe_shift_expression,
        $.equal_expression,
        $.identity_expression,
        $.rel_expression,
        $.in_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#EqualExpression
    equal_expression: $ =>
      seq(
        $._maybe_shift_expression,
        choice(
          "==",
          "!=",
        ),
        $._maybe_shift_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#IdentityExpression
    identity_expression: $ =>
      seq(
        $._maybe_shift_expression,
        optional(
          "!",
        ),
        "is",
        $._maybe_shift_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#RelExpression
    rel_expression: $ =>
      seq(
        $._maybe_shift_expression,
        choice(
          "<",
          "<=",
          ">",
          ">=",
        ),
        $._maybe_shift_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#InExpression
    in_expression: $ =>
      seq(
        $._maybe_shift_expression,
        optional(
          "!",
        ),
        "in",
        $._maybe_shift_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#ShiftExpression
    _maybe_shift_expression: $ =>
      choice(
        $._maybe_add_expression,
        $.shift_expression,
      ),

    shift_expression: $ =>
      seq(
        $._maybe_shift_expression,
        choice(
          "<<",
          ">>",
          ">>>",
        ),
        $._maybe_add_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#AddExpression
    _maybe_add_expression: $ =>
      choice(
        $._maybe_mul_expression,
        $.cat_expression,
        $.add_expression,
      ),

    add_expression: $ =>
      seq(
        $._maybe_add_expression,
        choice(
          "+",
          "-",
        ),
        $._maybe_mul_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#CatExpression
    cat_expression: $ =>
      seq(
        $._maybe_add_expression,
        "~",
        $._maybe_mul_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#MulExpression
    _maybe_mul_expression: $ =>
      choice(
        $._maybe_unary_expression,
        $.mul_expression,
      ),

    mul_expression: $ =>
      seq(
        $._maybe_mul_expression,
        choice(
          "*",
          "/",
          "%",
        ),
        $._maybe_unary_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#UnaryExpression
    _maybe_unary_expression: $ =>
      choice(
        $.complement_expression,
        $.delete_expression,
        $.cast_expression,
        $._maybe_pow_expression,
        $.unary_expression,
      ),

    unary_expression: $ =>
      seq(
        choice(
          "&",
          "++",
          "--",
          "*",
          "-",
          "+",
          "!",
        ),
        $._maybe_unary_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#ComplementExpression
    complement_expression: $ =>
      seq(
        "~",
        $._maybe_unary_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#NewExpression
    _maybe_new_expression: $ =>
      choice(
        $.new_anon_class_expression,
        $.new_expression,
      ),

    new_expression: $ =>
      seq(
        "new",
        optional(
          $.allocator_arguments,
        ),
        $.type,
        optional(
          choice(
            seq(
              "[",
              $._maybe_assign_expression,
              "]",
            ),
            seq(
              "(",
              optional(
                $.argument_list,
              ),
              ")",
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/expression.html#AllocatorArguments
    allocator_arguments: $ =>
      seq(
        "(",
        optional(
          $.argument_list,
        ),
        ")",
      ),

    // https://dlang.org/spec/expression.html#ArgumentList
    argument_list: $ =>
      seq(
        repeat(
          seq(
            $._maybe_assign_expression,
            ",",
          ),
        ),
        $._maybe_assign_expression,
        optional(
          ",",
        ),
      ),

    // ---

    // https://dlang.org/spec/expression.html#DeleteExpression
    delete_expression: $ =>
      seq(
        "delete",
        $._maybe_unary_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#CastExpression
    cast_expression: $ =>
      seq(
        "cast",
        "(",
        choice(
          $.type,
          optional(
            $.type_ctors,
          ),
        ),
        ")",
        $._maybe_unary_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#PowExpression
    _maybe_pow_expression: $ =>
      choice(
        $._maybe_postfix_expression,
        $.pow_expression,
      ),

    pow_expression: $ =>
      seq(
        $._maybe_postfix_expression,
        "^^",
        $._maybe_unary_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#PostfixExpression
    _maybe_postfix_expression: $ =>
      choice(
        $.primary_expression,
        $.index_expression,
        $.slice_expression,
        $.postfix_expression,
      ),

    postfix_expression: $ =>
      choice(
        seq(
          $._maybe_postfix_expression,
          choice(
            seq(
              ".",
              choice(
                $.identifier,
                $.template_instance,
                $._maybe_new_expression,
              ),
            ),
            "++",
            "--",
          ),
        ),
        seq(
          choice(
            $._maybe_postfix_expression,
            seq(
              optional(
                $.type_ctors,
              ),
              $._maybe_basic_type,
            ),
          ),
          "(",
          optional(
            $.argument_list,
          ),
          ")",
        ),
      ),

    // ---

    // https://dlang.org/spec/expression.html#IndexExpression
    index_expression: $ =>
      seq(
        $._maybe_postfix_expression,
        "[",
        $.argument_list,
        "]",
      ),

    // ---

    // https://dlang.org/spec/expression.html#SliceExpression
    slice_expression: $ =>
      seq(
        $._maybe_postfix_expression,
        "[",
        optional(
          seq(
            $.slice,
            optional(
              ",",
            ),
          ),
        ),
        "]",
      ),

    // https://dlang.org/spec/expression.html#Slice
    slice: $ =>
      seq(
        $._maybe_assign_expression,
        optional(
          seq(
            "..",
            $._maybe_assign_expression,
          ),
        ),
        optional(
          seq(
            ",",
            $.slice,
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/expression.html#PrimaryExpression
    primary_expression: $ =>
      choice(
        seq(
          optional(
            seq(
              optional(
                seq(
                  "(",
                  $.type,
                  ")",
                ),
              ),
              ".",
            ),
          ),
          choice(
            $.identifier,
            $.template_instance,
          ),
        ),
        "this",
        "super",
        "null",
        "true",
        "false",
        "$",
        $.integer_literal,
        $.float_literal,
        $.character_literal,
        $.string_literals,
        $.array_literal,
        $.assoc_array_literal,
        $._maybe_function_literal,
        $.assert_expression,
        $.mixin_expression,
        $.import_expression,
        $._maybe_new_expression,
        seq(
          choice(
            $.fundamental_type,
            seq(
              $.type_ctor,
              "(",
              $.type,
              ")",
            ),
          ),
          choice(
            seq(
              ".",
              $.identifier,
            ),
            seq(
              "(",
              optional(
                $.argument_list,
              ),
              ")",
            ),
          ),
        ),
        $.typeof,
        $.typeid_expression,
        $.is_expression,
        seq(
          "(",
          $.expression,
          ")",
        ),
        $.special_keyword,
        $.traits_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#StringLiterals
    string_literals: $ =>
      repeat1(
        $._string_literal,
      ),

    // ---

    // https://dlang.org/spec/expression.html#ArrayLiteral
    array_literal: $ =>
      seq(
        "[",
        optional(
          $.argument_list,
        ),
        "]",
      ),

    // ---

    // https://dlang.org/spec/expression.html#AssocArrayLiteral
    assoc_array_literal: $ =>
      seq(
        "[",
        $.key_value_pairs,
        "]",
      ),

    // https://dlang.org/spec/expression.html#KeyValuePairs
    key_value_pairs: $ =>
      seq(
        repeat(
          seq(
            $.key_value_pair,
            ",",
          ),
        ),
        $.key_value_pair,
        optional(
          ",",
        ),
      ),

    // https://dlang.org/spec/expression.html#KeyValuePair
    key_value_pair: $ =>
      seq(
        $.key_expression,
        ":",
        $.value_expression,
      ),

    // https://dlang.org/spec/expression.html#KeyExpression
    key_expression: $ =>
      $._maybe_assign_expression,

    // https://dlang.org/spec/expression.html#ValueExpression
    value_expression: $ =>
      $._maybe_assign_expression,

    // ---

    // https://dlang.org/spec/expression.html#FunctionLiteral
    _maybe_function_literal: $ =>
      choice(
        $.function_literal_body,
        $.function_literal,
      ),

    function_literal: $ =>
      choice(
        seq(
          choice(
            seq(
              "function",
              optional(
                "ref",
              ),
              optional(
                $.type,
              ),
              optional(
                $.parameter_with_attributes,
              ),
            ),
            seq(
              "delegate",
              optional(
                "ref",
              ),
              optional(
                $.type,
              ),
              optional(
                $.parameter_with_member_attributes,
              ),
            ),
            seq(
              optional(
                "ref",
              ),
              $.parameter_with_member_attributes,
            ),
          ),
          $._maybe_function_literal_body2,
        ),
        seq(
          $.identifier,
          "=>",
          $._maybe_assign_expression,
        ),
      ),

    // https://dlang.org/spec/expression.html#ParameterWithAttributes
    parameter_with_attributes: $ =>
      seq(
        $.parameters,
        optional(
          $.function_attributes,
        ),
      ),

    // https://dlang.org/spec/expression.html#ParameterWithMemberAttributes
    parameter_with_member_attributes: $ =>
      seq(
        $.parameters,
        optional(
          $.member_function_attributes,
        ),
      ),

    // https://dlang.org/spec/expression.html#FunctionLiteralBody2
    _maybe_function_literal_body2: $ =>
      choice(
        $.function_literal_body,
        $.function_literal_body2,
      ),

    function_literal_body2: $ =>
      seq(
        "=>",
        $._maybe_assign_expression,
      ),

    // ---

    // https://dlang.org/spec/expression.html#AssertExpression
    assert_expression: $ =>
      seq(
        "assert",
        "(",
        $.assert_arguments,
        ")",
      ),

    // https://dlang.org/spec/expression.html#AssertArguments
    assert_arguments: $ =>
      seq(
        $._maybe_assign_expression,
        optional(
          seq(
            ",",
            $._maybe_assign_expression,
          ),
        ),
        optional(
          ",",
        ),
      ),

    // ---

    // https://dlang.org/spec/expression.html#MixinExpression
    mixin_expression: $ =>
      seq(
        "mixin",
        "(",
        $.argument_list,
        ")",
      ),

    // ---

    // https://dlang.org/spec/expression.html#ImportExpression
    import_expression: $ =>
      seq(
        "import",
        "(",
        $._maybe_assign_expression,
        ")",
      ),

    // ---

    // https://dlang.org/spec/expression.html#TypeidExpression
    typeid_expression: $ =>
      seq(
        "typeid",
        "(",
        choice(
          $.type,
          $.expression,
        ),
        ")",
      ),

    // ---

    // https://dlang.org/spec/expression.html#IsExpression
    is_expression: $ =>
      seq(
        "is",
        "(",
        $.type,
        optional(
          $.identifier,
        ),
        optional(
          seq(
            choice(
              ":",
              "==",
            ),
            $.type_specialization,
            optional(
              seq(
                ",",
                $.template_parameter_list,
              ),
            ),
          ),
        ),
        ")",
      ),

    // https://dlang.org/spec/expression.html#TypeSpecialization
    type_specialization: $ =>
      choice(
        $.type,
        "struct",
        "union",
        "class",
        "interface",
        "enum",
        "__vector",
        "function",
        "delegate",
        "super",
        "const",
        "immutable",
        "inout",
        "shared",
        "return",
        "__parameters",
        "module",
        "package",
      ),

    // ---

    // https://dlang.org/spec/expression.html#SpecialKeyword
    special_keyword: $ =>
      choice(
        "__FILE__",
        "__FILE_FULL_PATH__",
        "__MODULE__",
        "__LINE__",
        "__FUNCTION__",
        "__PRETTY_FUNCTION__",
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/statement.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/statement.html#Statement
    _statement: $ =>
      choice(
        $.empty_statement,
        $._non_empty_statement,
        $.scope_block_statement,
      ),

    // https://dlang.org/spec/statement.html#EmptyStatement
    empty_statement: $ =>
      ";",

    // https://dlang.org/spec/statement.html#NoScopeNonEmptyStatement
    _no_scope_non_empty_statement: $ =>
      choice(
        $._non_empty_statement,
        $.block_statement,
      ),

    // https://dlang.org/spec/statement.html#NoScopeStatement
    _no_scope_statement: $ =>
      choice(
        $.empty_statement,
        $._non_empty_statement,
        $.block_statement,
      ),

    // https://dlang.org/spec/statement.html#NonEmptyOrScopeBlockStatement
    _non_empty_or_scope_block_statement: $ =>
      choice(
        $._non_empty_statement,
        $.scope_block_statement,
      ),

    // https://dlang.org/spec/statement.html#NonEmptyStatement
    _non_empty_statement: $ =>
      choice(
        $._non_empty_statement_no_case_no_default,
        $.case_statement,
        $.case_range_statement,
        $.default_statement,
      ),

    // https://dlang.org/spec/statement.html#NonEmptyStatementNoCaseNoDefault
    _non_empty_statement_no_case_no_default: $ =>
      choice(
        $.labeled_statement,
        $.expression_statement,
        $.declaration_statement,
        $.if_statement,
        $.while_statement,
        $.do_statement,
        $.for_statement,
        $.foreach_statement,
        $.switch_statement,
        $.final_switch_statement,
        $.continue_statement,
        $.break_statement,
        $.return_statement,
        $.goto_statement,
        $.with_statement,
        $.synchronized_statement,
        $.try_statement,
        $.scope_guard_statement,
        $.throw_statement,
        $._maybe_asm_statement,
        $.mixin_statement,
        $.foreach_range_statement,
        $.pragma_statement,
        $.conditional_statement,
        $.static_foreach_statement,
        $.static_assert,
        $.template_mixin,
        $.import_declaration,
      ),

    // ---

    // https://dlang.org/spec/statement.html#ScopeStatement
    _scope_statement: $ =>
      choice(
        $._non_empty_statement,
        $.block_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#ScopeBlockStatement
    scope_block_statement: $ =>
      $.block_statement,

    // ---

    // https://dlang.org/spec/statement.html#LabeledStatement
    labeled_statement: $ =>
      seq(
        $.identifier,
        ":",
        optional(
          $._statement,
        ),
      ),

    // ---

    // https://dlang.org/spec/statement.html#BlockStatement
    block_statement: $ =>
      seq(
        "{",
        optional(
          $.statement_list,
        ),
        "}",
      ),

    // https://dlang.org/spec/statement.html#StatementList
    statement_list: $ =>
      repeat1(
        $._statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#ExpressionStatement
    expression_statement: $ =>
      seq(
        $.expression,
        ";",
      ),

    // ---

    // https://dlang.org/spec/statement.html#DeclarationStatement
    declaration_statement: $ =>
      seq(
        optional(
          $.storage_classes,
        ),
        $._declaration,
      ),

    // ---

    // https://dlang.org/spec/statement.html#IfStatement
    if_statement: $ =>
      seq(
        "if",
        "(",
        $._maybe_if_condition,
        ")",
        $.then_statement,
        optional(
          seq(
            "else",
            $.else_statement,
          ),
        ),
      ),

    // https://dlang.org/spec/statement.html#IfCondition
    _maybe_if_condition: $ =>
      choice(
        $.expression,
        $.if_condition,
      ),

    if_condition: $ =>
      seq(
        choice(
          seq(
            choice(
              "auto",
              $.type_ctors,
            ),
            $.identifier,
          ),
          seq(
            optional(
              $.type_ctors,
            ),
            $._maybe_basic_type,
            $._declarator,
          ),
        ),
        "=",
        $.expression,
      ),

    // https://dlang.org/spec/statement.html#ThenStatement
    then_statement: $ =>
      $._scope_statement,

    // https://dlang.org/spec/statement.html#ElseStatement
    else_statement: $ =>
      $._scope_statement,

    // ---

    // https://dlang.org/spec/statement.html#WhileStatement
    while_statement: $ =>
      seq(
        "while",
        "(",
        $._maybe_if_condition,
        ")",
        $._scope_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#DoStatement
    do_statement: $ =>
      seq(
        "do",
        $._scope_statement,
        "while",
        "(",
        $.expression,
        ")",
        ";",
      ),

    // ---

    // https://dlang.org/spec/statement.html#ForStatement
    for_statement: $ =>
      seq(
        "for",
        "(",
        $._maybe_initialize,
        optional(
          $.test,
        ),
        ";",
        optional(
          $.increment,
        ),
        ")",
        $._scope_statement,
      ),

    // https://dlang.org/spec/statement.html#Initialize
    _maybe_initialize: $ =>
      choice(
        $._no_scope_non_empty_statement,
        $.initialize,
      ),

    initialize: $ =>
      ";",

    // https://dlang.org/spec/statement.html#Test
    test: $ =>
      $.expression,

    // https://dlang.org/spec/statement.html#Increment
    increment: $ =>
      $.expression,

    // ---

    // https://dlang.org/spec/statement.html#AggregateForeach
    aggregate_foreach: $ =>
      seq(
        $.foreach,
        "(",
        $.foreach_type_list,
        ";",
        $.foreach_aggregate,
        ")",
      ),

    // https://dlang.org/spec/statement.html#ForeachStatement
    foreach_statement: $ =>
      seq(
        $.aggregate_foreach,
        $._no_scope_non_empty_statement,
      ),

    // https://dlang.org/spec/statement.html#Foreach
    foreach: $ =>
      choice(
        "foreach",
        "foreach_reverse",
      ),

    // https://dlang.org/spec/statement.html#ForeachTypeList
    foreach_type_list: $ =>
      seq(
        repeat(
          seq(
            $.foreach_type,
            ",",
          ),
        ),
        $.foreach_type,
      ),

    // https://dlang.org/spec/statement.html#ForeachType
    foreach_type: $ =>
      seq(
        optional(
          $.foreach_type_attributes,
        ),
        choice(
          seq(
            $._maybe_basic_type,
            $._declarator,
          ),
          seq(
            optional(
              "alias",
            ),
            $.identifier,
          ),
        ),
      ),

    // https://dlang.org/spec/statement.html#ForeachTypeAttributes
    foreach_type_attributes: $ =>
      repeat1(
        $._maybe_foreach_type_attribute,
      ),

    // https://dlang.org/spec/statement.html#ForeachTypeAttribute
    _maybe_foreach_type_attribute: $ =>
      choice(
        $.type_ctor,
        $.foreach_type_attribute,
      ),

    foreach_type_attribute: $ =>
      choice(
        "ref",
        "enum",
      ),

    // https://dlang.org/spec/statement.html#ForeachAggregate
    foreach_aggregate: $ =>
      $.expression,

    // ---

    // https://dlang.org/spec/statement.html#RangeForeach
    range_foreach: $ =>
      seq(
        $.foreach,
        "(",
        $.foreach_type,
        ";",
        $.lwr_expression,
        "..",
        $.upr_expression,
        ")",
      ),

    // https://dlang.org/spec/statement.html#LwrExpression
    lwr_expression: $ =>
      $.expression,

    // https://dlang.org/spec/statement.html#UprExpression
    upr_expression: $ =>
      $.expression,

    // https://dlang.org/spec/statement.html#ForeachRangeStatement
    foreach_range_statement: $ =>
      seq(
        $.range_foreach,
        $._scope_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#SwitchStatement
    switch_statement: $ =>
      seq(
        "switch",
        "(",
        $.expression,
        ")",
        $._scope_statement,
      ),

    // https://dlang.org/spec/statement.html#CaseStatement
    case_statement: $ =>
      seq(
        "case",
        $.argument_list,
        ":",
        optional(
          $.scope_statement_list,
        ),
      ),

    // https://dlang.org/spec/statement.html#CaseRangeStatement
    case_range_statement: $ =>
      seq(
        "case",
        $.first_exp,
        ":",
        "..",
        "case",
        $.last_exp,
        ":",
        optional(
          $.scope_statement_list,
        ),
      ),

    // https://dlang.org/spec/statement.html#FirstExp
    first_exp: $ =>
      $._maybe_assign_expression,

    // https://dlang.org/spec/statement.html#LastExp
    last_exp: $ =>
      $._maybe_assign_expression,

    // https://dlang.org/spec/statement.html#DefaultStatement
    default_statement: $ =>
      seq(
        "default",
        ":",
        optional(
          $.scope_statement_list,
        ),
      ),

    // https://dlang.org/spec/statement.html#ScopeStatementList
    scope_statement_list: $ =>
      $.statement_list_no_case_no_default,

    // https://dlang.org/spec/statement.html#StatementListNoCaseNoDefault
    statement_list_no_case_no_default: $ =>
      repeat1(
        $._statement_no_case_no_default,
      ),

    // https://dlang.org/spec/statement.html#StatementNoCaseNoDefault
    _statement_no_case_no_default: $ =>
      choice(
        $.empty_statement,
        $._non_empty_statement_no_case_no_default,
        $.scope_block_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#FinalSwitchStatement
    final_switch_statement: $ =>
      seq(
        "final",
        "switch",
        "(",
        $.expression,
        ")",
        $._scope_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#ContinueStatement
    continue_statement: $ =>
      seq(
        "continue",
        optional(
          $.identifier,
        ),
        ";",
      ),

    // ---

    // https://dlang.org/spec/statement.html#BreakStatement
    break_statement: $ =>
      seq(
        "break",
        optional(
          $.identifier,
        ),
        ";",
      ),

    // ---

    // https://dlang.org/spec/statement.html#ReturnStatement
    return_statement: $ =>
      seq(
        "return",
        optional(
          $.expression,
        ),
        ";",
      ),

    // ---

    // https://dlang.org/spec/statement.html#GotoStatement
    goto_statement: $ =>
      seq(
        "goto",
        choice(
          $.identifier,
          "default",
          seq(
            "case",
            optional(
              $.expression,
            ),
          ),
        ),
        ";",
      ),

    // ---

    // https://dlang.org/spec/statement.html#WithStatement
    with_statement: $ =>
      seq(
        "with",
        "(",
        choice(
          $.expression,
          $.symbol,
          $.template_instance,
        ),
        ")",
        $._scope_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#SynchronizedStatement
    synchronized_statement: $ =>
      seq(
        "synchronized",
        optional(
          seq(
            "(",
            $.expression,
            ")",
          ),
        ),
        $._scope_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#TryStatement
    try_statement: $ =>
      seq(
        "try",
        $._scope_statement,
        choice(
          seq(
            $.catches,
            optional(
              $.finally_statement,
            ),
          ),
          $.finally_statement,
        ),
      ),

    // https://dlang.org/spec/statement.html#Catches
    catches: $ =>
      repeat1(
        $.catch,
      ),

    // https://dlang.org/spec/statement.html#Catch
    catch: $ =>
      seq(
        "catch",
        "(",
        $.catch_parameter,
        ")",
        $._no_scope_non_empty_statement,
      ),

    // https://dlang.org/spec/statement.html#CatchParameter
    catch_parameter: $ =>
      seq(
        $._maybe_basic_type,
        optional(
          $.identifier,
        ),
      ),

    // https://dlang.org/spec/statement.html#FinallyStatement
    finally_statement: $ =>
      seq(
        "finally",
        $._no_scope_non_empty_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#ThrowStatement
    throw_statement: $ =>
      seq(
        "throw",
        $.expression,
        ";",
      ),

    // ---

    // https://dlang.org/spec/statement.html#ScopeGuardStatement
    scope_guard_statement: $ =>
      seq(
        "scope",
        "(",
        choice(
          "exit",
          "success",
          "failure",
        ),
        ")",
        $._non_empty_or_scope_block_statement,
      ),

    // ---

    // https://dlang.org/spec/statement.html#AsmStatement
    _maybe_asm_statement: $ =>
      choice(
        $.gcc_asm_statement,
        $.asm_statement,
      ),

    asm_statement: $ =>
      seq(
        "asm",
        optional(
          $.function_attributes,
        ),
        "{",
        optional(
          $.asm_instruction_list,
        ),
        "}",
      ),

    // https://dlang.org/spec/statement.html#AsmInstructionList
    asm_instruction_list: $ =>
      repeat1(
        seq(
          optional(
            $.asm_instruction,
          ),
          ";",
        ),
      ),

    // ---

    // https://dlang.org/spec/statement.html#MixinStatement
    mixin_statement: $ =>
      seq(
        "mixin",
        "(",
        $.argument_list,
        ")",
        ";",
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/struct.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/struct.html#AggregateDeclaration
    _aggregate_declaration: $ =>
      choice(
        $._maybe_class_declaration,
        $._maybe_interface_declaration,
        $._maybe_struct_declaration,
        $._maybe_union_declaration,
      ),

    // https://dlang.org/spec/struct.html#StructDeclaration
    _maybe_struct_declaration: $ =>
      choice(
        $.struct_template_declaration,
        $.anon_struct_declaration,
        $.struct_declaration,
      ),

    struct_declaration: $ =>
      seq(
        "struct",
        $.identifier,
        choice(
          ";",
          $.aggregate_body,
        ),
      ),

    // https://dlang.org/spec/struct.html#AnonStructDeclaration
    anon_struct_declaration: $ =>
      seq(
        "struct",
        $.aggregate_body,
      ),

    // https://dlang.org/spec/struct.html#UnionDeclaration
    _maybe_union_declaration: $ =>
      choice(
        $.union_template_declaration,
        $.anon_union_declaration,
        $.union_declaration,
      ),

    union_declaration: $ =>
      seq(
        "union",
        $.identifier,
        choice(
          ";",
          $.aggregate_body,
        ),
      ),

    // https://dlang.org/spec/struct.html#AnonUnionDeclaration
    anon_union_declaration: $ =>
      seq(
        "union",
        $.aggregate_body,
      ),

    // https://dlang.org/spec/struct.html#AggregateBody
    aggregate_body: $ =>
      seq(
        "{",
        optional(
          $.decl_defs,
        ),
        "}",
      ),

    // ---

    // https://dlang.org/spec/struct.html#Postblit
    postblit: $ =>
      seq(
        "this",
        "(",
        "this",
        ")",
        optional(
          $.member_function_attributes,
        ),
        choice(
          ";",
          $._function_body,
        ),
      ),

    // ---

    // https://dlang.org/spec/struct.html#Invariant
    invariant: $ =>
      seq(
        "invariant",
        choice(
          seq(
            "(",
            choice(
              seq(
                ")",
                $.block_statement,
              ),
              seq(
                $.assert_arguments,
                ")",
                ";",
              ),
            ),
          ),
          $.block_statement,
        ),
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/class.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/class.html#ClassDeclaration
    _maybe_class_declaration: $ =>
      choice(
        $.class_template_declaration,
        $.class_declaration,
      ),

    class_declaration: $ =>
      seq(
        "class",
        $.identifier,
        choice(
          ";",
          seq(
            optional(
              $.base_class_list,
            ),
            $.aggregate_body,
          ),
        ),
      ),

    // https://dlang.org/spec/class.html#BaseClassList
    base_class_list: $ =>
      seq(
        ":",
        $.super_class_or_interface,
        optional(
          seq(
            ",",
            $.interfaces,
          ),
        ),
      ),

    // https://dlang.org/spec/class.html#SuperClassOrInterface
    super_class_or_interface: $ =>
      $._maybe_basic_type,

    // https://dlang.org/spec/class.html#Interfaces
    interfaces: $ =>
      seq(
        repeat(
          seq(
            $.interface,
            ",",
          ),
        ),
        $.interface,
      ),

    // https://dlang.org/spec/class.html#Interface
    interface: $ =>
      $._maybe_basic_type,

    // ---

    // https://dlang.org/spec/class.html#Constructor
    _maybe_constructor: $ =>
      choice(
        $.constructor_template,
        $.constructor,
      ),

    constructor: $ =>
      seq(
        "this",
        $.parameters,
        optional(
          $.member_function_attributes,
        ),
        $._function_body,
      ),

    // ---

    // https://dlang.org/spec/class.html#Destructor
    destructor: $ =>
      seq(
        "~",
        "this",
        "(",
        ")",
        optional(
          $.member_function_attributes,
        ),
        $._function_body,
      ),

    // ---

    // https://dlang.org/spec/class.html#StaticConstructor
    static_constructor: $ =>
      seq(
        "static",
        "this",
        "(",
        ")",
        optional(
          $.member_function_attributes,
        ),
        choice(
          ";",
          $._function_body,
        ),
      ),

    // ---

    // https://dlang.org/spec/class.html#StaticDestructor
    static_destructor: $ =>
      seq(
        "static",
        "~",
        "this",
        "(",
        ")",
        optional(
          $.member_function_attributes,
        ),
        choice(
          ";",
          $._function_body,
        ),
      ),

    // ---

    // https://dlang.org/spec/class.html#SharedStaticConstructor
    shared_static_constructor: $ =>
      seq(
        "shared",
        "static",
        "this",
        "(",
        ")",
        optional(
          $.member_function_attributes,
        ),
        choice(
          ";",
          $._function_body,
        ),
      ),

    // ---

    // https://dlang.org/spec/class.html#SharedStaticDestructor
    shared_static_destructor: $ =>
      seq(
        "shared",
        "static",
        "~",
        "this",
        "(",
        ")",
        optional(
          $.member_function_attributes,
        ),
        choice(
          ";",
          $._function_body,
        ),
      ),

    // ---

    // https://dlang.org/spec/class.html#Allocator
    allocator: $ =>
      seq(
        "new",
        $.parameters,
        $._function_body,
      ),

    // ---

    // https://dlang.org/spec/class.html#Deallocator
    deallocator: $ =>
      seq(
        "delete",
        $.parameters,
        $._function_body,
      ),

    // ---

    // https://dlang.org/spec/class.html#AliasThis
    alias_this: $ =>
      seq(
        "alias",
        $.identifier,
        "this",
        ";",
      ),

    // ---

    // https://dlang.org/spec/class.html#NewAnonClassExpression
    new_anon_class_expression: $ =>
      seq(
        "new",
        optional(
          $.allocator_arguments,
        ),
        "class",
        optional(
          $.constructor_args,
        ),
        optional(
          $.super_class_or_interface,
        ),
        optional(
          $.interfaces,
        ),
        $.aggregate_body,
      ),

    // https://dlang.org/spec/class.html#ConstructorArgs
    constructor_args: $ =>
      seq(
        "(",
        optional(
          $.argument_list,
        ),
        ")",
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/interface.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/interface.html#InterfaceDeclaration
    _maybe_interface_declaration: $ =>
      choice(
        $.interface_template_declaration,
        $.interface_declaration,
      ),

    interface_declaration: $ =>
      seq(
        "interface",
        $.identifier,
        choice(
          ";",
          seq(
            optional(
              $.base_interface_list,
            ),
            $.aggregate_body,
          ),
        ),
      ),

    // https://dlang.org/spec/interface.html#BaseInterfaceList
    base_interface_list: $ =>
      seq(
        ":",
        $.interfaces,
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/enum.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/enum.html#EnumDeclaration
    _maybe_enum_declaration: $ =>
      choice(
        $.anonymous_enum_declaration,
        $.enum_declaration,
      ),

    enum_declaration: $ =>
      seq(
        "enum",
        $.identifier,
        optional(
          seq(
            ":",
            $.enum_base_type,
          ),
        ),
        $.enum_body,
      ),

    // https://dlang.org/spec/enum.html#EnumBaseType
    enum_base_type: $ =>
      $.type,

    // https://dlang.org/spec/enum.html#EnumBody
    enum_body: $ =>
      choice(
        seq(
          "{",
          $.enum_members,
          "}",
        ),
        ";",
      ),

    // https://dlang.org/spec/enum.html#EnumMembers
    enum_members: $ =>
      seq(
        repeat(
          seq(
            $.enum_member,
            ",",
          ),
        ),
        $.enum_member,
        optional(
          ",",
        ),
      ),

    // https://dlang.org/spec/enum.html#EnumMemberAttributes
    enum_member_attributes: $ =>
      repeat1(
        $._maybe_enum_member_attribute,
      ),

    // https://dlang.org/spec/enum.html#EnumMemberAttribute
    _maybe_enum_member_attribute: $ =>
      choice(
        $.deprecated_attribute,
        $.user_defined_attribute,
        $.enum_member_attribute,
      ),

    enum_member_attribute: $ =>
      seq(
        "@",
        "disable",
      ),

    // https://dlang.org/spec/enum.html#EnumMember
    enum_member: $ =>
      seq(
        optional(
          $.enum_member_attributes,
        ),
        $.identifier,
        optional(
          seq(
            "=",
            $._maybe_assign_expression,
          ),
        ),
      ),

    // https://dlang.org/spec/enum.html#AnonymousEnumDeclaration
    anonymous_enum_declaration: $ =>
      seq(
        "enum",
        choice(
          seq(
            optional(
              seq(
                ":",
                $.enum_base_type,
              ),
            ),
            "{",
            $.enum_members,
          ),
          seq(
            "{",
            $.anonymous_enum_members,
          ),
        ),
        "}",
      ),

    // https://dlang.org/spec/enum.html#AnonymousEnumMembers
    anonymous_enum_members: $ =>
      seq(
        repeat(
          seq(
            $._maybe_anonymous_enum_member,
            ",",
          ),
        ),
        $._maybe_anonymous_enum_member,
        optional(
          ",",
        ),
      ),

    // https://dlang.org/spec/enum.html#AnonymousEnumMember
    _maybe_anonymous_enum_member: $ =>
      choice(
        $.enum_member,
        $.anonymous_enum_member,
      ),

    anonymous_enum_member: $ =>
      seq(
        $.type,
        $.identifier,
        "=",
        $._maybe_assign_expression,
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/function.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/function.html#FuncDeclaration
    _maybe_func_declaration: $ =>
      choice(
        $.auto_func_declaration,
        $.func_declaration,
      ),

    func_declaration: $ =>
      seq(
        optional(
          $.storage_classes,
        ),
        $._maybe_basic_type,
        $.func_declarator,
        $._function_body,
      ),

    // https://dlang.org/spec/function.html#AutoFuncDeclaration
    auto_func_declaration: $ =>
      seq(
        $.storage_classes,
        $.identifier,
        $.func_declarator_suffix,
        $._function_body,
      ),

    // https://dlang.org/spec/function.html#FuncDeclarator
    func_declarator: $ =>
      seq(
        optional(
          $.type_suffixes,
        ),
        $.identifier,
        $.func_declarator_suffix,
      ),

    // https://dlang.org/spec/function.html#FuncDeclaratorSuffix
    func_declarator_suffix: $ =>
      choice(
        seq(
          $.parameters,
          optional(
            $.member_function_attributes,
          ),
        ),
        seq(
          $.template_parameters,
          $.parameters,
          optional(
            $.member_function_attributes,
          ),
          optional(
            $.constraint,
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/function.html#Parameters
    parameters: $ =>
      seq(
        "(",
        optional(
          $.parameter_list,
        ),
        ")",
      ),

    // https://dlang.org/spec/function.html#ParameterList
    parameter_list: $ =>
      seq(
        repeat(
          seq(
            $.parameter,
            ",",
          ),
        ),
        choice(
          seq(
            $.parameter,
            optional(
              ",",
            ),
          ),
          seq(
            optional(
              $.variadic_arguments_attributes,
            ),
            "...",
          ),
        ),
      ),

    // https://dlang.org/spec/function.html#Parameter
    parameter: $ =>
      seq(
        optional(
          $.parameter_attributes,
        ),
        choice(
          seq(
            $._maybe_basic_type,
            $._declarator,
          ),
          $.type,
        ),
        optional(
          seq(
            "=",
            $._maybe_assign_expression,
          ),
        ),
        optional(
          "...",
        ),
      ),

    // https://dlang.org/spec/function.html#ParameterAttributes
    parameter_attributes: $ =>
      repeat1(
        choice(
          $._maybe_in_out,
          $.user_defined_attribute,
        ),
      ),

    // https://dlang.org/spec/function.html#InOut
    _maybe_in_out: $ =>
      choice(
        $.type_ctor,
        $.in_out,
      ),

    in_out: $ =>
      choice(
        "auto",
        "final",
        "in",
        "lazy",
        "out",
        "ref",
        "return",
        "scope",
      ),

    // https://dlang.org/spec/function.html#VariadicArgumentsAttributes
    variadic_arguments_attributes: $ =>
      repeat1(
        $.variadic_arguments_attribute,
      ),

    // https://dlang.org/spec/function.html#VariadicArgumentsAttribute
    variadic_arguments_attribute: $ =>
      choice(
        "const",
        "immutable",
        "return",
        "scope",
        "shared",
      ),

    // ---

    // https://dlang.org/spec/function.html#FunctionAttributes
    function_attributes: $ =>
      repeat1(
        $._function_attribute,
      ),

    // https://dlang.org/spec/function.html#FunctionAttribute
    _function_attribute: $ =>
      choice(
        $.function_attribute_kwd,
        $.property,
        $._maybe_at_attribute,
      ),

    // https://dlang.org/spec/function.html#MemberFunctionAttributes
    member_function_attributes: $ =>
      repeat1(
        $._maybe_member_function_attribute,
      ),

    // https://dlang.org/spec/function.html#MemberFunctionAttribute
    _maybe_member_function_attribute: $ =>
      choice(
        $._function_attribute,
        $.member_function_attribute,
      ),

    member_function_attribute: $ =>
      choice(
        "const",
        "immutable",
        "inout",
        "return",
        "scope",
        "shared",
      ),

    // ---

    // https://dlang.org/spec/function.html#FunctionBody
    _function_body: $ =>
      choice(
        $.specified_function_body,
        $.missing_function_body,
        $.shortened_function_body,
      ),

    // https://dlang.org/spec/function.html#FunctionLiteralBody
    function_literal_body: $ =>
      $.specified_function_body,

    // https://dlang.org/spec/function.html#SpecifiedFunctionBody
    specified_function_body: $ =>
      seq(
        choice(
          seq(
            optional(
              seq(
                optional(
                  $.function_contracts,
                ),
                $._in_out_contract_expression,
              ),
            ),
            optional(
              "do",
            ),
          ),
          seq(
            optional(
              $.function_contracts,
            ),
            $._in_out_statement,
            "do",
          ),
        ),
        $.block_statement,
      ),

    // https://dlang.org/spec/function.html#MissingFunctionBody
    missing_function_body: $ =>
      choice(
        ";",
        seq(
          optional(
            $.function_contracts,
          ),
          choice(
            seq(
              $._in_out_contract_expression,
              ";",
            ),
            $._in_out_statement,
          ),
        ),
      ),

    // https://dlang.org/spec/function.html#ShortenedFunctionBody
    shortened_function_body: $ =>
      seq(
        optional(
          seq(
            optional(
              $.function_contracts,
            ),
            $._in_out_contract_expression,
          ),
        ),
        "=>",
        $._maybe_assign_expression,
        ";",
      ),

    // ---

    // https://dlang.org/spec/function.html#FunctionContracts
    function_contracts: $ =>
      repeat1(
        $._function_contract,
      ),

    // https://dlang.org/spec/function.html#FunctionContract
    _function_contract: $ =>
      choice(
        $._in_out_contract_expression,
        $._in_out_statement,
      ),

    // https://dlang.org/spec/function.html#InOutContractExpression
    _in_out_contract_expression: $ =>
      choice(
        $.in_contract_expression,
        $.out_contract_expression,
      ),

    // https://dlang.org/spec/function.html#InOutStatement
    _in_out_statement: $ =>
      choice(
        $.in_statement,
        $.out_statement,
      ),

    // https://dlang.org/spec/function.html#InContractExpression
    in_contract_expression: $ =>
      seq(
        "in",
        "(",
        $.assert_arguments,
        ")",
      ),

    // https://dlang.org/spec/function.html#OutContractExpression
    out_contract_expression: $ =>
      seq(
        "out",
        "(",
        optional(
          $.identifier,
        ),
        ";",
        $.assert_arguments,
        ")",
      ),

    // https://dlang.org/spec/function.html#InStatement
    in_statement: $ =>
      seq(
        "in",
        $.block_statement,
      ),

    // https://dlang.org/spec/function.html#OutStatement
    out_statement: $ =>
      seq(
        "out",
        optional(
          seq(
            "(",
            $.identifier,
            ")",
          ),
        ),
        $.block_statement,
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/template.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/template.html#TemplateDeclaration
    template_declaration: $ =>
      seq(
        "template",
        $.identifier,
        $.template_parameters,
        optional(
          $.constraint,
        ),
        "{",
        optional(
          $.decl_defs,
        ),
        "}",
      ),

    // https://dlang.org/spec/template.html#TemplateParameters
    template_parameters: $ =>
      seq(
        "(",
        optional(
          $.template_parameter_list,
        ),
        ")",
      ),

    // https://dlang.org/spec/template.html#TemplateParameterList
    template_parameter_list: $ =>
      seq(
        repeat(
          seq(
            $._template_parameter,
            ",",
          ),
        ),
        $._template_parameter,
        optional(
          ",",
        ),
      ),

    // https://dlang.org/spec/template.html#TemplateParameter
    _template_parameter: $ =>
      choice(
        $.template_type_parameter,
        $.template_value_parameter,
        $.template_alias_parameter,
        $.template_sequence_parameter,
        $.template_this_parameter,
      ),

    // ---

    // https://dlang.org/spec/template.html#TemplateInstance
    template_instance: $ =>
      seq(
        $.identifier,
        $.template_arguments,
      ),

    // https://dlang.org/spec/template.html#TemplateArguments
    template_arguments: $ =>
      seq(
        "!",
        choice(
          seq(
            "(",
            optional(
              $.template_argument_list,
            ),
            ")",
          ),
          $.template_single_argument,
        ),
      ),

    // https://dlang.org/spec/template.html#TemplateArgumentList
    template_argument_list: $ =>
      seq(
        repeat(
          seq(
            $._template_argument,
            ",",
          ),
        ),
        $._template_argument,
        optional(
          ",",
        ),
      ),

    // https://dlang.org/spec/template.html#TemplateArgument
    _template_argument: $ =>
      choice(
        $.type,
        $._maybe_assign_expression,
        $.symbol,
      ),

    // https://dlang.org/spec/template.html#Symbol
    symbol: $ =>
      seq(
        optional(
          ".",
        ),
        $.symbol_tail,
      ),

    // https://dlang.org/spec/template.html#SymbolTail
    symbol_tail: $ =>
      seq(
        choice(
          $.identifier,
          $.template_instance,
        ),
        optional(
          seq(
            ".",
            $.symbol_tail,
          ),
        ),
      ),

    // https://dlang.org/spec/template.html#TemplateSingleArgument
    template_single_argument: $ =>
      choice(
        $.identifier,
        $.fundamental_type,
        $.character_literal,
        $._string_literal,
        $.integer_literal,
        $.float_literal,
        "true",
        "false",
        "null",
        "this",
        $.special_keyword,
      ),

    // ---

    // https://dlang.org/spec/template.html#TemplateTypeParameter
    template_type_parameter: $ =>
      seq(
        $.identifier,
        optional(
          $.template_type_parameter_specialization,
        ),
        optional(
          $.template_type_parameter_default,
        ),
      ),

    // https://dlang.org/spec/template.html#TemplateTypeParameterSpecialization
    template_type_parameter_specialization: $ =>
      seq(
        ":",
        $.type,
      ),

    // https://dlang.org/spec/template.html#TemplateTypeParameterDefault
    template_type_parameter_default: $ =>
      seq(
        "=",
        $.type,
      ),

    // ---

    // https://dlang.org/spec/template.html#TemplateThisParameter
    template_this_parameter: $ =>
      seq(
        "this",
        $.template_type_parameter,
      ),

    // ---

    // https://dlang.org/spec/template.html#TemplateValueParameter
    template_value_parameter: $ =>
      seq(
        $._maybe_basic_type,
        $._declarator,
        optional(
          $.template_value_parameter_specialization,
        ),
        optional(
          $.template_value_parameter_default,
        ),
      ),

    // https://dlang.org/spec/template.html#TemplateValueParameterSpecialization
    template_value_parameter_specialization: $ =>
      seq(
        ":",
        $._maybe_conditional_expression,
      ),

    // https://dlang.org/spec/template.html#TemplateValueParameterDefault
    template_value_parameter_default: $ =>
      seq(
        "=",
        choice(
          $._maybe_assign_expression,
          $.special_keyword,
        ),
      ),

    // ---

    // https://dlang.org/spec/template.html#TemplateAliasParameter
    template_alias_parameter: $ =>
      seq(
        "alias",
        choice(
          $.identifier,
          seq(
            $._maybe_basic_type,
            $._declarator,
          ),
        ),
        optional(
          $.template_alias_parameter_specialization,
        ),
        optional(
          $.template_alias_parameter_default,
        ),
      ),

    // https://dlang.org/spec/template.html#TemplateAliasParameterSpecialization
    template_alias_parameter_specialization: $ =>
      seq(
        ":",
        choice(
          $.type,
          $._maybe_conditional_expression,
        ),
      ),

    // https://dlang.org/spec/template.html#TemplateAliasParameterDefault
    template_alias_parameter_default: $ =>
      seq(
        "=",
        choice(
          $.type,
          $._maybe_conditional_expression,
        ),
      ),

    // ---

    // https://dlang.org/spec/template.html#TemplateSequenceParameter
    template_sequence_parameter: $ =>
      seq(
        $.identifier,
        "...",
      ),

    // ---

    // https://dlang.org/spec/template.html#ConstructorTemplate
    constructor_template: $ =>
      seq(
        "this",
        $.template_parameters,
        $.parameters,
        optional(
          $.member_function_attributes,
        ),
        optional(
          $.constraint,
        ),
        choice(
          ":",
          $._function_body,
        ),
      ),

    // ---

    // https://dlang.org/spec/template.html#ClassTemplateDeclaration
    class_template_declaration: $ =>
      seq(
        "class",
        $.identifier,
        $.template_parameters,
        choice(
          ";",
          seq(
            choice(
              seq(
                optional(
                  $.constraint,
                ),
                optional(
                  $.base_class_list,
                ),
              ),
              seq(
                optional(
                  $.base_class_list,
                ),
                optional(
                  $.constraint,
                ),
              ),
            ),
            $.aggregate_body,
          ),
        ),
      ),

    // https://dlang.org/spec/template.html#InterfaceTemplateDeclaration
    interface_template_declaration: $ =>
      seq(
        "interface",
        $.identifier,
        $.template_parameters,
        choice(
          ";",
          seq(
            choice(
              seq(
                optional(
                  $.base_interface_list,
                ),
                optional(
                  $.constraint,
                ),
              ),
              seq(
                $.constraint,
                $.base_interface_list,
              ),
            ),
            $.aggregate_body,
          ),
        ),
      ),

    // https://dlang.org/spec/template.html#StructTemplateDeclaration
    struct_template_declaration: $ =>
      seq(
        "struct",
        $.identifier,
        $.template_parameters,
        choice(
          ";",
          seq(
            optional(
              $.constraint,
            ),
            $.aggregate_body,
          ),
        ),
      ),

    // https://dlang.org/spec/template.html#UnionTemplateDeclaration
    union_template_declaration: $ =>
      seq(
        "union",
        $.identifier,
        $.template_parameters,
        choice(
          ";",
          seq(
            optional(
              $.constraint,
            ),
            $.aggregate_body,
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/template.html#Constraint
    constraint: $ =>
      seq(
        "if",
        "(",
        $.expression,
        ")",
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/template-mixin.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/template-mixin.html#TemplateMixinDeclaration
    template_mixin_declaration: $ =>
      seq(
        "mixin",
        "template",
        $.identifier,
        $.template_parameters,
        optional(
          $.constraint,
        ),
        "{",
        optional(
          $.decl_defs,
        ),
        "}",
      ),

    // https://dlang.org/spec/template-mixin.html#TemplateMixin
    template_mixin: $ =>
      seq(
        "mixin",
        $._maybe_mixin_template_name,
        optional(
          $.template_arguments,
        ),
        optional(
          $.identifier,
        ),
        ";",
      ),

    // https://dlang.org/spec/template-mixin.html#MixinTemplateName
    _maybe_mixin_template_name: $ =>
      choice(
        $.mixin_qualified_identifier,
        $.mixin_template_name,
      ),

    mixin_template_name: $ =>
      seq(
        optional(
          $.typeof,
        ),
        ".",
        $.mixin_qualified_identifier,
      ),

    // https://dlang.org/spec/template-mixin.html#MixinQualifiedIdentifier
    mixin_qualified_identifier: $ =>
      choice(
        $.identifier,
        seq(
          choice(
            $.identifier,
            $.template_instance,
          ),
          ".",
          $.mixin_qualified_identifier,
        ),
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/version.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/version.html#ConditionalDeclaration
    conditional_declaration: $ =>
      seq(
        $._condition,
        optional(
          seq(
            $._maybe_declaration_block,
            "else",
          ),
        ),
        choice(
          $._maybe_declaration_block,
          seq(
            ":",
            optional(
              $.decl_defs,
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/version.html#ConditionalStatement
    conditional_statement: $ =>
      seq(
        $._condition,
        $._no_scope_non_empty_statement,
        optional(
          seq(
            "else",
            $._no_scope_non_empty_statement,
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/version.html#Condition
    _condition: $ =>
      choice(
        $.version_condition,
        $.debug_condition,
        $.static_if_condition,
      ),

    // ---

    // https://dlang.org/spec/version.html#VersionCondition
    version_condition: $ =>
      seq(
        "version",
        "(",
        choice(
          $.integer_literal,
          $.identifier,
          "unittest",
          "assert",
        ),
        ")",
      ),

    // ---

    // https://dlang.org/spec/version.html#VersionSpecification
    version_specification: $ =>
      seq(
        "version",
        "=",
        choice(
          $.identifier,
          $.integer_literal,
        ),
        ";",
      ),

    // ---

    // https://dlang.org/spec/version.html#DebugCondition
    debug_condition: $ =>
      seq(
        "debug",
        optional(
          seq(
            "(",
            choice(
              $.integer_literal,
              $.identifier,
            ),
            ")",
          ),
        ),
      ),

    // ---

    // https://dlang.org/spec/version.html#DebugSpecification
    debug_specification: $ =>
      seq(
        "debug",
        "=",
        choice(
          $.identifier,
          $.integer_literal,
        ),
        ";",
      ),

    // ---

    // https://dlang.org/spec/version.html#StaticIfCondition
    static_if_condition: $ =>
      seq(
        "static",
        "if",
        "(",
        $._maybe_assign_expression,
        ")",
      ),

    // ---

    // https://dlang.org/spec/version.html#StaticForeach
    static_foreach: $ =>
      seq(
        "static",
        choice(
          $.aggregate_foreach,
          $.range_foreach,
        ),
      ),

    // https://dlang.org/spec/version.html#StaticForeachDeclaration
    static_foreach_declaration: $ =>
      seq(
        $.static_foreach,
        choice(
          $._maybe_declaration_block,
          seq(
            ":",
            optional(
              $.decl_defs,
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/version.html#StaticForeachStatement
    static_foreach_statement: $ =>
      seq(
        $.static_foreach,
        $._no_scope_non_empty_statement,
      ),

    // ---

    // https://dlang.org/spec/version.html#StaticAssert
    static_assert: $ =>
      seq(
        "static",
        "assert",
        "(",
        $.assert_arguments,
        ")",
        ";",
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/traits.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/traits.html#TraitsExpression
    traits_expression: $ =>
      seq(
        "__traits",
        "(",
        $.traits_keyword,
        optional(
          seq(
            ",",
            $.traits_arguments,
          ),
        ),
        ")",
      ),

    // https://dlang.org/spec/traits.html#TraitsKeyword
    traits_keyword: $ =>
      choice(
        "isAbstractClass",
        "isArithmetic",
        "isAssociativeArray",
        "isFinalClass",
        "isPOD",
        "isNested",
        "isFuture",
        "isDeprecated",
        "isFloating",
        "isIntegral",
        "isScalar",
        "isStaticArray",
        "isUnsigned",
        "isDisabled",
        "isVirtualFunction",
        "isVirtualMethod",
        "isAbstractFunction",
        "isFinalFunction",
        "isStaticFunction",
        "isOverrideFunction",
        "isTemplate",
        "isRef",
        "isOut",
        "isLazy",
        "isReturnOnStack",
        "isCopyable",
        "isZeroInit",
        "isModule",
        "isPackage",
        "hasMember",
        "hasCopyConstructor",
        "hasPostblit",
        "identifier",
        "getAliasThis",
        "getAttributes",
        "getFunctionAttributes",
        "getFunctionVariadicStyle",
        "getLinkage",
        "getLocation",
        "getMember",
        "getOverloads",
        "getParameterStorageClasses",
        "getPointerBitmap",
        "getCppNamespaces",
        "getVisibility",
        "getProtection",
        "getTargetInfo",
        "getVirtualFunctions",
        "getVirtualMethods",
        "getUnitTests",
        "parent",
        "child",
        "classInstanceSize",
        "getVirtualIndex",
        "allMembers",
        "derivedMembers",
        "isSame",
        "compiles",
        "toType",
      ),

    // https://dlang.org/spec/traits.html#TraitsArguments
    traits_arguments: $ =>
      seq(
        repeat(
          seq(
            $._traits_argument,
            ",",
          ),
        ),
        $._traits_argument,
      ),

    // https://dlang.org/spec/traits.html#TraitsArgument
    _traits_argument: $ =>
      choice(
        $._maybe_assign_expression,
        $.type,
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/unittest.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/unittest.html#UnitTest
    unit_test: $ =>
      seq(
        "unittest",
        $.block_statement,
      ),

    // ------------------------------------------------------------------------
    // https://dlang.org/spec/iasm.html
    // ------------------------------------------------------------------------

    // https://dlang.org/spec/iasm.html#AsmInstruction
    asm_instruction: $ =>
      seq(
        repeat(
          seq(
            $.identifier,
            ":",
          ),
        ),
        choice(
          seq(
            $.identifier,
            ":",
          ),
          seq(
            "align",
            $._integer_expression,
          ),
          "even",
          "naked",
          seq(
            choice(
              "db",
              "ds",
              "di",
              "dl",
              "df",
              "dw",
              "dd",
              "dq",
              "de",
              $.opcode,
            ),
            $.operands,
          ),
          seq(
            choice(
              "db",
              "ds",
              "di",
              "dl",
              "dw",
              "dq",
            ),
            $._string_literal,
          ),
          $.opcode,
        ),
      ),

    // https://dlang.org/spec/iasm.html#Opcode
    opcode: $ =>
      choice(
        $.identifier,
        "int",
        "in",
        "out",
      ),

    // https://dlang.org/spec/iasm.html#Operands
    operands: $ =>
      seq(
        repeat(
          seq(
            $.operand,
            ",",
          ),
        ),
        $.operand,
      ),

    // ---

    // https://dlang.org/spec/iasm.html#IntegerExpression
    _integer_expression: $ =>
      choice(
        $.integer_literal,
        $.identifier,
      ),

    // ---

    // https://dlang.org/spec/iasm.html#Register
    register: $ =>
      choice(
        "AL",
        "AH",
        "AX",
        "EAX",
        "BL",
        "BH",
        "BX",
        "EBX",
        "CL",
        "CH",
        "CX",
        "ECX",
        "DL",
        "DH",
        "DX",
        "EDX",
        "BP",
        "EBP",
        "SP",
        "ESP",
        "DI",
        "EDI",
        "SI",
        "ESI",
        "ES",
        "CS",
        "SS",
        "DS",
        "GS",
        "FS",
        "CR0",
        "CR2",
        "CR3",
        "CR4",
        "DR0",
        "DR1",
        "DR2",
        "DR3",
        "DR6",
        "DR7",
        "TR3",
        "TR4",
        "TR5",
        "TR6",
        "TR7",
        "ST",
        "ST(0)",
        "ST(1)",
        "ST(2)",
        "ST(3)",
        "ST(4)",
        "ST(5)",
        "ST(6)",
        "ST(7)",
        "MM0",
        "MM1",
        "MM2",
        "MM3",
        "MM4",
        "MM5",
        "MM6",
        "MM7",
        "XMM0",
        "XMM1",
        "XMM2",
        "XMM3",
        "XMM4",
        "XMM5",
        "XMM6",
        "XMM7",
      ),

    // ---

    // https://dlang.org/spec/iasm.html#Register64
    register64: $ =>
      choice(
        "RAX",
        "RBX",
        "RCX",
        "RDX",
        "BPL",
        "RBP",
        "SPL",
        "RSP",
        "DIL",
        "RDI",
        "SIL",
        "RSI",
        "R8B",
        "R8W",
        "R8D",
        "R8",
        "R9B",
        "R9W",
        "R9D",
        "R9",
        "R10B",
        "R10W",
        "R10D",
        "R10",
        "R11B",
        "R11W",
        "R11D",
        "R11",
        "R12B",
        "R12W",
        "R12D",
        "R12",
        "R13B",
        "R13W",
        "R13D",
        "R13",
        "R14B",
        "R14W",
        "R14D",
        "R14",
        "R15B",
        "R15W",
        "R15D",
        "R15",
        "XMM8",
        "XMM9",
        "XMM10",
        "XMM11",
        "XMM12",
        "XMM13",
        "XMM14",
        "XMM15",
        "YMM0",
        "YMM1",
        "YMM2",
        "YMM3",
        "YMM4",
        "YMM5",
        "YMM6",
        "YMM7",
        "YMM8",
        "YMM9",
        "YMM10",
        "YMM11",
        "YMM12",
        "YMM13",
        "YMM14",
        "YMM15",
      ),

    // ---

    // https://dlang.org/spec/iasm.html#Operand
    operand: $ =>
      $._maybe_asm_exp,

    // https://dlang.org/spec/iasm.html#AsmExp
    _maybe_asm_exp: $ =>
      choice(
        $._maybe_asm_log_or_exp,
        $.asm_exp,
      ),

    asm_exp: $ =>
      seq(
        $._maybe_asm_log_or_exp,
        "?",
        $._maybe_asm_exp,
        ":",
        $._maybe_asm_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmLogOrExp
    _maybe_asm_log_or_exp: $ =>
      choice(
        $._maybe_asm_log_and_exp,
        $.asm_log_or_exp,
      ),

    asm_log_or_exp: $ =>
      seq(
        $._maybe_asm_log_or_exp,
        "||",
        $._maybe_asm_log_and_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmLogAndExp
    _maybe_asm_log_and_exp: $ =>
      choice(
        $._maybe_asm_or_exp,
        $.asm_log_and_exp,
      ),

    asm_log_and_exp: $ =>
      seq(
        $._maybe_asm_log_and_exp,
        "&&",
        $._maybe_asm_or_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmOrExp
    _maybe_asm_or_exp: $ =>
      choice(
        $._maybe_asm_xor_exp,
        $.asm_or_exp,
      ),

    asm_or_exp: $ =>
      seq(
        $._maybe_asm_or_exp,
        "|",
        $._maybe_asm_xor_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmXorExp
    _maybe_asm_xor_exp: $ =>
      choice(
        $._maybe_asm_and_exp,
        $.asm_xor_exp,
      ),

    asm_xor_exp: $ =>
      seq(
        $._maybe_asm_xor_exp,
        "^",
        $._maybe_asm_and_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmAndExp
    _maybe_asm_and_exp: $ =>
      choice(
        $._maybe_asm_equal_exp,
        $.asm_and_exp,
      ),

    asm_and_exp: $ =>
      seq(
        $._maybe_asm_and_exp,
        "&",
        $._maybe_asm_equal_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmEqualExp
    _maybe_asm_equal_exp: $ =>
      choice(
        $._maybe_asm_rel_exp,
        $.asm_equal_exp,
      ),

    asm_equal_exp: $ =>
      seq(
        $._maybe_asm_equal_exp,
        choice(
          "==",
          "!=",
        ),
        $._maybe_asm_rel_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmRelExp
    _maybe_asm_rel_exp: $ =>
      choice(
        $._maybe_asm_shift_exp,
        $.asm_rel_exp,
      ),

    asm_rel_exp: $ =>
      seq(
        $._maybe_asm_rel_exp,
        choice(
          "<",
          "<=",
          ">",
          ">=",
        ),
        $._maybe_asm_shift_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmShiftExp
    _maybe_asm_shift_exp: $ =>
      choice(
        $._maybe_asm_add_exp,
        $.asm_shift_exp,
      ),

    asm_shift_exp: $ =>
      seq(
        $._maybe_asm_shift_exp,
        choice(
          "<<",
          ">>",
          ">>>",
        ),
        $._maybe_asm_add_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmAddExp
    _maybe_asm_add_exp: $ =>
      choice(
        $._maybe_asm_mul_exp,
        $.asm_add_exp,
      ),

    asm_add_exp: $ =>
      seq(
        $._maybe_asm_add_exp,
        choice(
          "+",
          "-",
        ),
        $._maybe_asm_mul_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmMulExp
    _maybe_asm_mul_exp: $ =>
      choice(
        $._maybe_asm_br_exp,
        $.asm_mul_exp,
      ),

    asm_mul_exp: $ =>
      seq(
        $._maybe_asm_mul_exp,
        choice(
          "*",
          "/",
          "%",
        ),
        $._maybe_asm_br_exp,
      ),

    // https://dlang.org/spec/iasm.html#AsmBrExp
    _maybe_asm_br_exp: $ =>
      choice(
        $._maybe_asm_una_exp,
        $.asm_br_exp,
      ),

    asm_br_exp: $ =>
      seq(
        $._maybe_asm_br_exp,
        "[",
        $._maybe_asm_exp,
        "]",
      ),

    // https://dlang.org/spec/iasm.html#AsmUnaExp
    _maybe_asm_una_exp: $ =>
      choice(
        $.asm_primary_exp,
        $.asm_una_exp,
      ),

    asm_una_exp: $ =>
      choice(
        seq(
          choice(
            $.asm_type_prefix,
            "offsetof",
            "seg",
            "short",
            "near",
            "far",
          ),
          $._maybe_asm_exp,
        ),
        seq(
          "[",
          $._maybe_asm_exp,
          "]",
        ),
        seq(
          choice(
            "+",
            "-",
            "!",
            "~",
          ),
          $._maybe_asm_una_exp,
        ),
      ),

    // https://dlang.org/spec/iasm.html#AsmPrimaryExp
    asm_primary_exp: $ =>
      choice(
        $.integer_literal,
        $.float_literal,
        "__LOCAL_SIZE",
        "$",
        seq(
          choice(
            $.register,
            $.register64,
          ),
          optional(
            seq(
              ":",
              $._maybe_asm_exp,
            ),
          ),
        ),
        $.dot_identifier,
        "this",
      ),

    // https://dlang.org/spec/iasm.html#DotIdentifier
    dot_identifier: $ =>
      choice(
        seq(
          $.identifier,
          optional(
            seq(
              ".",
              $.dot_identifier,
            ),
          ),
        ),
        seq(
          $.fundamental_type,
          ".",
          $.identifier,
        ),
      ),

    // ---

    // https://dlang.org/spec/iasm.html#AsmTypePrefix
    asm_type_prefix: $ =>
      seq(
        choice(
          "near",
          "far",
          "word",
          "dword",
          "qword",
          $.fundamental_type,
        ),
        "ptr",
      ),

    // ---

    // https://dlang.org/spec/iasm.html#GccAsmStatement
    gcc_asm_statement: $ =>
      seq(
        "asm",
        optional(
          $.function_attributes,
        ),
        "{",
        $.gcc_asm_instruction_list,
        "}",
      ),

    // https://dlang.org/spec/iasm.html#GccAsmInstructionList
    gcc_asm_instruction_list: $ =>
      repeat1(
        seq(
          optional(
            $._gcc_asm_instruction,
          ),
          ";",
        ),
      ),

    // https://dlang.org/spec/iasm.html#GccAsmInstruction
    _gcc_asm_instruction: $ =>
      choice(
        $.gcc_basic_asm_instruction,
        $.gcc_ext_asm_instruction,
        $.gcc_goto_asm_instruction,
      ),

    // https://dlang.org/spec/iasm.html#GccBasicAsmInstruction
    gcc_basic_asm_instruction: $ =>
      $._maybe_assign_expression,

    // https://dlang.org/spec/iasm.html#GccExtAsmInstruction
    gcc_ext_asm_instruction: $ =>
      seq(
        $._maybe_assign_expression,
        ":",
        optional(
          $.gcc_asm_operands,
        ),
        optional(
          seq(
            ":",
            optional(
              $.gcc_asm_operands,
            ),
            optional(
              seq(
                ":",
                optional(
                  $._maybe_gcc_asm_clobbers,
                ),
              ),
            ),
          ),
        ),
      ),

    // https://dlang.org/spec/iasm.html#GccGotoAsmInstruction
    gcc_goto_asm_instruction: $ =>
      seq(
        $._maybe_assign_expression,
        ":",
        ":",
        optional(
          $.gcc_asm_operands,
        ),
        ":",
        optional(
          $._maybe_gcc_asm_clobbers,
        ),
        ":",
        optional(
          $.gcc_asm_goto_labels,
        ),
      ),

    // https://dlang.org/spec/iasm.html#GccAsmOperands
    gcc_asm_operands: $ =>
      seq(
        optional(
          $.gcc_symbolic_name,
        ),
        $._string_literal,
        "(",
        $._maybe_assign_expression,
        ")",
        optional(
          seq(
            ",",
            $.gcc_asm_operands,
          ),
        ),
      ),

    // https://dlang.org/spec/iasm.html#GccSymbolicName
    gcc_symbolic_name: $ =>
      seq(
        "[",
        $.identifier,
        "]",
      ),

    // https://dlang.org/spec/iasm.html#GccAsmClobbers
    _maybe_gcc_asm_clobbers: $ =>
      choice(
        $._string_literal,
        $.gcc_asm_clobbers,
      ),

    gcc_asm_clobbers: $ =>
      seq(
        $._string_literal,
        ",",
        $._maybe_gcc_asm_clobbers,
      ),

    // https://dlang.org/spec/iasm.html#GccAsmGotoLabels
    gcc_asm_goto_labels: $ =>
      seq(
        $.identifier,
        optional(
          seq(
            ",",
            $.gcc_asm_goto_labels,
          ),
        ),
      ),
  }
});
