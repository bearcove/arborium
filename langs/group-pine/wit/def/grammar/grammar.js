/**
 * @file WIT grammar for tree-sitter
 * @author Liam Woodleigh-Hardinge <liam.woodleigh@gmail.com>
 * @license Apache-2.0 WITH LLVM-exception
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const commaSeparatedList = (rule) =>
  seq(rule, repeat(seq(',', rule)), optional(','));

module.exports = grammar({
  name: 'wit',

  extras: $ => [
    /\s/,
    $.line_comment,
    $.block_comment,
  ],

  externals: $ => [
    $._block_comment_content,
    $._block_doc_comment_marker,
    $._error_sentinel,
    $._line_doc_content,
  ],

  supertypes: $ => [
    $._gate_item,
    $._typedef_item,
  ],

  rules: {
    source_file: ($) =>
      seq(
        repeat($._statement),
      ),

    _statement: $ => choice(
      $.package_decl,
      $.toplevel_use_item,
      $.world_item,
      $.interface_item,
    ),


    // URI patterns shared between `use_path` and `package_decl`
    _uri_head: $ => seq($.id, ':'),
    _uri_tail: $ => seq('/', $.id),
    _version: $ => seq('@', alias($._valid_semver, $.version)),

    package_decl: ($) =>
      seq(
        'package',
        repeat1($._uri_head),
        $.id,
        repeat($._uri_tail),
        optional($._version),
        ';',
      ),

    toplevel_use_item: ($) =>
      seq(
        'use',
        $.use_path,
        optional(seq('as', field('alias', $.id))),
        ';',
      ),

    use_path: ($) =>
      choice(
        $.id,
        seq(
          repeat1($._uri_head),
          $.id,
          repeat1($._uri_tail),
          optional($._version),
        )),

    // See here: https://github.com/WebAssembly/component-model/blob/f44d2377f79ea6dd105060f08f01e269cda7df85/design/mvp/WIT.md#wit-identifiers
    // And here: https://github.com/WebAssembly/component-model/blob/c182ca92143c06287e71c3d1125e38d49ffc32b3/design/mvp/Explainer.md#import-and-export-definitions
    id: ($) =>
      /%?(([a-z][a-z0-9]*|[A-Z][A-Z0-9]*))(-([a-z][a-z0-9]*|[A-Z][A-Z0-9]*))*/,

    // As per https://semver.org/, this regex allows for trailing metadata after MAJOR.MINOR.PATCH
    _valid_semver: ($) =>
      /(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?/,

    world_item: ($) =>
      seq(optional($._gate), 'world', field('name', $.id), alias($._world_body, $.body)),

    _world_body: ($) =>
      seq('{', repeat($._world_items), '}'),

    _world_items: ($) =>
      seq(
        optional($._gate),
        choice(
          $.export_item,
          $.import_item,
          $.use_item,
          $._typedef_item,
          $.include_item,
        )),

    export_item: ($) =>
      choice(
        seq('export', field('name', $.id), ':', $.extern_type),
        seq('export', $.use_path, ';'),
      ),

    import_item: ($) =>
      choice(
        seq('import', field('name', $.id), ':', $.extern_type),
        seq('import', $.use_path, ';'),
      ),

    extern_type: ($) =>
      choice(seq($.func_type, ';'), seq('interface', alias($._interface_body, $.body))),

    include_item: ($) =>
      choice(
        seq('include', $.use_path, ';'),
        seq(
          'include',
          $.use_path,
          'with',
          alias($._include_names_body, $.definitions),
        ),
      ),

    _include_names_body: ($) =>
      seq('{', $._include_names_list, '}'),

    _include_names_list: ($) =>
      commaSeparatedList( $.include_names_item),
    include_names_item: ($) => $.alias_item,

    alias_item: $ =>
      seq(field('path', $.id), 'as', field('alias', $.id)),
    include_names_item: ($) => seq($.id, 'as', $.id),

    interface_item: ($) =>
      seq(optional($._gate), 'interface', field('name', $.id), alias($._interface_body, $.body)),

    _interface_body: ($) =>
      seq('{', repeat($._interface_items), '}'),

    _interface_items: ($) =>
      seq(
        optional($._gate),
        choice(
          $._typedef_item,
          $.use_item,
          $.func_item,
        )),

    _typedef_item: ($) =>
      choice(
        $.resource_item,
        $.variant_items,
        $.record_item,
        $.flags_items,
        $.enum_items,
        $.type_item,
      ),

    func_item: ($) => seq(field('name', $.id), ':', $.func_type, ';'),

    func_type: ($) =>
      seq(
        optional('async'),
        'func',
        $.param_list,
        optional($.result_list),
      ),

    param_list: ($) => seq('(', optional($._named_type_list), ')'),

    result_list: ($) =>
      choice(seq('->', $.ty), seq('->', '(', optional($._named_type_list), ')')),

    _named_type_list: ($) => commaSeparatedList($.named_type),

    named_type: ($) => seq(field('name', $.id), ':', field('type', $.ty)),

    use_item: ($) =>
      seq('use', $.use_path, '.',
        alias($._use_names_body, $.definitions),
        ';'),

    _use_names_body: $ =>
      seq('{', $._use_names_list, '}'),

    _use_names_list: ($) =>
      commaSeparatedList($.use_names_item),

    use_names_item: ($) => choice($.id, $.alias_item),

    type_item: ($) =>
      seq('type', field('alias', $.id), '=', field('type', $.ty), ';'),

    record_item: ($) =>
      seq('record', field('name', $.id), alias($._record_body, $.body)),
    _record_body: ($) => seq('{', $._record_fields, '}'),
    _record_fields: ($) => commaSeparatedList($.record_field),
    record_field: ($) => seq(field('name', $.id), ':', field('type', $.ty)),

    flags_items: ($) => seq('flags', field('name', $.id), alias($._flags_body, $.body)),
    _flags_body: ($) => seq('{', $._flags_fields, '}'),
    _flags_fields: ($) => commaSeparatedList(alias($.id, $.flags_field)),

    variant_items: ($) =>
      seq('variant', field('name', $.id), alias($._variant_body, $.body)),
    _variant_body: ($) =>
      seq('{', $._variant_cases, '}'),

    _variant_cases: ($) => commaSeparatedList($.variant_case),
    variant_case: ($) =>
      choice(
        field('name', $.id),
        seq(field('name', $.id), '(', field('type', $.ty), ')'),
      ),

    enum_items: ($) => seq('enum', field('name', $.id), alias($._enum_body, $.body)),
    _enum_body: ($) => seq('{', $._enum_cases, '}'),
    _enum_cases: ($) => commaSeparatedList(alias($.id, $.enum_case)),

    resource_item: ($) =>
      seq(
        'resource',
        field('name', $.id),
        choice(';', optional(alias($._resource_body, $.body))),
      ),

    _resource_body: ($) => seq('{', repeat($.resource_method), '}'),

    resource_method: ($) =>
      choice(
        $.func_item,
        seq($.id, ':', 'static', $.func_type, ';'),
        seq('constructor', $.param_list, ';'),
      ),

    ty: ($) =>
      prec(
        1,
        choice(
          'u8',
          'u16',
          'u32',
          'u64',
          's8',
          's16',
          's32',
          's64',
          'f32',
          'f64',
          'char',
          'bool',
          'string',
          $.tuple,
          $.list,
          $.option,
          $.result,
          $.id,
          $.handle,
          $.future,
          $.stream,
        ),
      ),

    tuple: ($) => seq('tuple', '<', $.tuple_list, '>'),

    tuple_list: ($) => commaSeparatedList($.ty),

    uint: _ =>/[1-9][0-9]*/,
    list: ($) => seq('list',
      '<',
      $.ty,
      optional(field('size', seq(',', $.uint))),
      '>',
    ),

    option: ($) => seq('option', '<', $.ty, '>'),

    result: ($) =>
      seq(
        'result',
        optional(
          choice(
            seq('<', $.ty, ',', $.ty, '>'),
            seq('<', '_', ',', $.ty, '>'),
            seq('<', $.ty, '>'),
          ),
        ),
      ),

    handle: ($) => prec(0, choice($.id, seq('borrow', '<', $.id, '>'))),
    future: ($) => seq('future', optional(seq('<', $.ty, '>'))),
    stream: ($) => seq('stream', optional(seq('<', $.ty, '>'))),


    comment: $ => choice(
      $.line_comment,
      $.block_comment,
    ),

    line_comment: $ => seq(
      // All line comments start with two //
      '//',
      // Then are followed by:
      // - 2 or more slashes making it a regular comment
      // - 1 slash or 1 or more bang operators making it a doc comment
      // - or just content for the comment
      choice(
        // A tricky edge case where what looks like a doc comment is not
        seq(token.immediate(prec(2, '//')), /.*/),
        // A line doc comment
        seq(token.immediate(prec(2, '/')), field('doc', alias($._line_doc_content, $.doc_comment))),
        // A regular comment
        token.immediate(prec(1, /.*/)),
      ),
    ),

    block_comment: $ => seq(
      '/*',
      optional(
        choice(
          // Documentation block comments: /** docs */
          seq(
            $._block_doc_comment_marker,
            optional(field('doc', alias($._block_comment_content, $.doc_comment))),
          ),
          // Non-doc block comments
          $._block_comment_content,
        ),
      ),
      '*/',
    ),
    // https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md#feature-gates
    // gate ::= gate-item*
    // gate-item ::= unstable-gate
    //             | since-gate
    //             | deprecated-gate
    //
    // unstable-gate ::= '@unstable' '(' feature-field ')'
    // since-gate ::= '@since' '(' version-field ')'
    // deprecated-gate ::= '@deprecated' '(' version-field ')'
    //
    // feature-field ::= 'feature' '=' id
    // version-field ::= 'version' '=' <valid semver>
    _gate: $ => repeat1($._gate_item),
    _gate_item: $ => choice(
      $.unstable_gate,
      $.since_gate,
      $.deprecated_gate,
    ),

    unstable_gate: $ => seq( '@', 'unstable', '(', $._feature_field, ')'),
    _feature_field: $ => seq('feature', '=', field('feature', $.id)),

    since_gate: $ => seq('@', 'since', '(', $._version_field, ')'),
    deprecated_gate: $ => seq('@', 'deprecated', '(', $._version_field, ')'),
    _version_field: $ => seq('version', '=', alias($._valid_semver, $.version)),
  },
});
