// @ts-check
/// <reference types="tree-sitter-cli/dsl" />

const PREC = {
  CONTROL: 0,
  ASSIGN: 1,
  TERNARY: 2,
  NULL_COALESCE: 3,
  LOGICAL_OR: 4,
  LOGICAL_AND: 5,
  BIT_OR: 6,
  BIT_XOR: 7,
  BIT_AND: 8,
  EQUALITY: 10,
  COMPARE: 10,
  IS: 10,
  IN: 10,
  SHIFT: 13,
  ADD: 14,
  MULTIPLY: 15,
  RANGE: 16,
  UNARY: 21,
  POSTFIX: 18,
  PRIMARY: 19,
  CALL: 20,
  // BLOCK: 26,
  OBJECT_DECL: 27,
  TYPE_ANNOTATION: 29,
  TYPE_PARAMS: 31,
  CONDITIONAL: 1001,
  MACRO: -1,
};

/**
 * @param {string|Rule} sep - The separator token or rule
 * @param {Rule} rule - The element rule
 * @returns {Rule}
 */
function sep1(sep, rule) {
  return seq(rule, repeat(seq(sep, rule)));
}
/**
 * @param {string|Rule} sep
 * @param {Rule} rule
 * @returns {Rule}
 */
function sep(sep, rule) {
  return optional(sep1(sep, rule));
}
/** @param {Rule} rule @returns {Rule} */
const commaSep1 = (rule) => sep1(",", rule);
/** @param {Rule} rule @returns {Rule} */
const commaSep = (rule) => sep(",", rule);
/** @param {Rule} rule @returns {Rule} */
const dotSep1 = (rule) => sep1(".", rule);
/** @param {Rule} rule @returns {Rule} */
const dotSep = (rule) => sep(".", rule);

const RESERVED_KEYWORDS = [
  "abstract",
  "as",
  "break",
  "case",
  "cast",
  "catch",
  "class",
  "continue",
  "default",
  "do",
  "dynamic",
  "else",
  "enum",
  "extends",
  "extern",
  "false",
  "final",
  "for",
  "function",
  "if",
  "implements",
  "import",
  "in",
  "inline",
  "interface",
  "macro",
  "new",
  "null",
  "override",
  "package",
  "private",
  "public",
  "return",
  "static",
  "super",
  "switch",
  "this",
  "throw",
  "true",
  "try",
  "typedef",
  "untyped",
  "using",
  "var",
  "while",
];

export default grammar({
  name: "haxe",
  extras: ($) => [/\s+/, $.comment, $.conditional],
  reserved: {
    global: (_) => RESERVED_KEYWORDS,
  },
  conflicts: ($) => [
    // `extern` is legal both as a TYPE modifier (`extern class Foo`) and as a
    // FIELD modifier (`extern private static inline var W:Int = 1024;`), so a
    // leading modifier run is ambiguous between opening a type declaration and
    // opening one of its members until the next keyword settles it.
    [$.ClassType, $.ClassVar, $.ClassMethod, $.DefType, $.EnumType],
    [
      $.ClassType,
      $.ClassVar,
      $.ClassMethod,
      $.DefType,
      $.EnumType,
      $._conditional_body,
    ],
    // A declarator in a var statement is ambiguous with itself: at `=` the
    // parser can take the initializer, or stop and let the enclosing EVars be
    // reduced so that `=` becomes an EBinop assignment. GLR has to carry both
    // or the third and later declarator in `var a = 1, b = 2, c = 3;` is lost.
    [$._var_declarator],
    [
      $.AbstractType,
      $.ClassMethod,
      $.ClassType,
      $.ClassVar,
      $.DefType,
      $.EnumType,
    ],
    [
      $.AbstractType,
      $.ClassMethod,
      $.ClassType,
      $.ClassVar,
      $.DefType,
      $.EnumType,
      $._conditional_body,
    ],
    [$.AbstractType, $.ClassType, $.DefType, $.EnumType],
    [$.AbstractType, $.ClassType, $._conditional_body],
    [$.AbstractType, $.ClassType],
    [$.AbstractType, $.DefType, $.EnumType, $._conditional_body],
    [$.AbstractType, $.DefType, $.EnumType],
    [$.ClassMethod, $._conditional_body],
    [$.ClassType, $.ClassMethod],
    [$.ClassType, $.ClassVar, $.ClassMethod, $._conditional_body],
    [$.ClassType, $.ClassVar, $.ClassMethod],
    [$.ClassType, $.DefType, $.EnumType, $._conditional_body],
    [$.ClassType, $.DefType, $.EnumType],
    [$.ClassType, $._conditional_body],
    [$.ClassVar, $.ClassMethod, $._conditional_body],
    [$.ClassVar, $.ClassMethod],
    [$.ClassVar, $._conditional_body],
    [$.DefType, $.EnumType, $._conditional_body],
    [$.EBinop],
    [$.EBlock, $.EObjectDecl],
    [$.ECall, $.EFunction, $.ClassMethod],
    [$.ECall],
    [$.EFunction],
    [$.TypePath],
    [$._EConst, $.FunctionArg, $.compile_condition],
    [$._EConst, $.FunctionArg],
    [$._EConst, $._expr_lhs, $.compile_condition],
    [$._EConst, $._expr_lhs],
    [$._EConst, $.compile_condition],
    [$._Expr, $._expr_lhs],
    [$._block_or_expr, $._comprehension_body],
    [$._block_or_expr],
    [$._class_field, $._conditional_body],
    [$._dot_path],
    [$._expr_atom, $._expr_value],
    [$._expr_lhs, $.compile_condition],
    [$._expr_meta, $.ECall],
    [$._expr_prim, $._expr_value],
    [$._expr_value, $._Expr],
    [$._expr_value, $._expr_lhs],
    [$._type_decl, $._class_field, $._conditional_body],
    [$._type_decl, $._conditional_body],
    [$.cases],
    [$.import, $._dot_path],
  ],
  inline: ($) => [$._semicolon, $.visibility, $.rest, $._expr_postfix],
  word: ($) => $.identifier,
  rules: {
    module: ($) =>
      seq(
        optional($.package),
        repeat(choice($.import, $.using)),
        repeat(choice($._type_decl, $._expr_statement)),
      ),

    //////////////////////////////////////////////////////////////////////////

    package: ($) =>
      seq("package", optional(field("path", $._dot_path)), $._semicolon),

    import: ($) =>
      seq(
        "import",
        choice(
          seq(
            optional(field("path", seq($._dot_path, "."))),
            field("module", $.type_name),
            optional(field("sub", repeat1(seq(".", $.identifier)))),
            optional(
              choice(
                seq(".", $.wildcard),
                seq("as", field("alias", choice($.type_name, $.identifier))),
              ),
            ),
          ),
          seq(
            field(
              "path",
              seq(
                dotSep1($.package_name),
                "as",
                field("alias", choice($.type_name, $.identifier)),
              ),
            ),
          ),
          seq(field("path", repeat1(seq($.package_name, "."))), $.wildcard),
        ),
        $._semicolon,
      ),

    using: ($) =>
      seq(
        "using",
        optional(field("path", seq($._dot_path, "."))),
        repeat(seq($.type_name, ".")),
        field("type", $.type_name),
        $._semicolon,
      ),

    ///////////////////////////////////////////////////////////////////////////

    _expr_statement: ($) => seq($._Expr, optional($._semicolon)),

    _Expr: ($) => choice($.EBinop, $.ETernary, $.EUnop, $._expr_postfix),
    _expr_postfix: ($) => choice($.ECall, $.EField, $.EArray, $._expr_prim),
    _expr_prim: ($) =>
      choice(
        $._expr_atom,
        $.EBreak,
        $.EContinue,
        $.EFor,
        $.EIf,
        $.EReturn,
        $.ESwitch,
        $.EThrow,
        $.ETry,
        $.EVars,
        $.EWhile,
        $._expr_meta,
        $.EBlock,
      ),
    _expr_atom: ($) =>
      choice($._EConst, $._EParenthesis, $.EObjectDecl, $.EArrayDecl, $.ENew),
    _expr_meta: ($) =>
      choice(
        $.EArrowFunction,
        $.ECast,
        $.ECheckType,
        $.EFunction,
        $.EMeta,
        $.EUntyped,
        //
        $.macro,
        $.reification,
        $.type_trace,
        $.wildcard_pattern,
      ),
    _expr_value: ($) =>
      choice(
        $._expr_postfix,
        $._expr_atom,
        $.EObjectDecl,
        $.EArrayDecl,
        $.ENew,
      ),
    _expr_lhs: ($) => choice($.identifier, $.EField, $.EArray),

    _block_or_expr: ($) =>
      choice(
        prec(1, $.EBlock),
        prec.dynamic(-1, seq($._Expr, optional($._semicolon))),
      ),

    ECall: ($) =>
      prec.left(
        PREC.CALL,
        seq(
          optional("inline"),
          field(
            "callee",
            choice(
              $._EParenthesis,
              // $.ECall, //TODO: should not be required
              $.EField,
              $.identifier,
              $.super,
            ),
          ),
          "(",
          field("args", commaSep(choice($.reification, $._Expr))),
          ")",
        ),
      ),
    EField: ($) =>
      prec.left(
        PREC.CALL,
        seq(
          field("object", $._Expr),
          field("op", choice(".", "?.")),
          optional("$"),
          field("name", $.identifier),
        ),
      ),
    EArray: ($) =>
      prec.left(
        PREC.CALL,
        seq(field("array", $._Expr), "[", field("index", $._Expr), "]"),
      ),
    _EConst: ($) =>
      choice(
        $.Int,
        $.Float,
        $.String,
        $.Regexp,
        $.true,
        $.false,
        $.null,
        $.this,
        $.super,
        $.identifier,
      ),
    _EParenthesis: ($) => prec(PREC.PRIMARY, seq("(", $._Expr, ")")),
    ENew: ($) =>
      prec.left(PREC.CALL, seq("new", $.TypePath, "(", commaSep($._Expr), ")")),
    EFunction: ($) =>
      prec.right(PREC.CONTROL - 1, seq(optional("inline"), $._function_decl)),
    EArrowFunction: ($) =>
      prec.right(
        PREC.CONTROL - 1,
        seq(
          choice(
            seq("(", ")"),
            field("args", $.identifier),
            seq(field("args", $._function_args), optional($._type_annotation)),
          ),
          "->",
          field("body", $._block_or_expr),
        ),
      ),
    EVars: ($) =>
      prec.right(
        PREC.ASSIGN,
        seq(
          choice("var", "final"),
          commaSep1($._var_declarator),
        ),
      ),
    _var_declarator: ($) =>
      seq(
        field("name", $.identifier),
        optional(field("type", $._type_annotation)),
        optional(seq("=", field("value", $._Expr))),
      ),
    ETernary: ($) =>
      prec.right(
        PREC.TERNARY,
        seq(
          field("cond", $._Expr),
          "?",
          field("if", $._Expr),
          ":",
          field("else", $._Expr),
        ),
      ),
    EBinop: ($) => {
      const BINOPS = [
        {
          ops: [
            "=>",
            "=",
            "+=",
            "-=",
            "*=",
            "/=",
            "%=",
            "&=",
            "|=",
            "^=",
            "<<=",
            ">>=",
            ">>>=",
          ],
          prec: PREC.ASSIGN,
          assoc: "right",
        },
        { ops: ["??"], prec: PREC.NULL_COALESCE, assoc: "right" },
        { ops: ["||"], prec: PREC.LOGICAL_OR, assoc: "left" },
        { ops: ["&&"], prec: PREC.LOGICAL_AND, assoc: "left" },
        { ops: ["|"], prec: PREC.BIT_OR, assoc: "left" },
        { ops: ["^"], prec: PREC.BIT_XOR, assoc: "left" },
        { ops: ["&"], prec: PREC.BIT_AND, assoc: "left" },
        { ops: ["==", "!="], prec: PREC.EQUALITY, assoc: "non" },
        { ops: [">", ">=", "<", "<="], prec: PREC.COMPARE, assoc: "non" },
        { ops: ["is"], prec: PREC.IS, rhs: $.TypePath, assoc: "non" },
        { ops: ["in"], prec: PREC.IN, assoc: "non" },
        { ops: ["<<", ">>", ">>>"], prec: PREC.SHIFT, assoc: "left" },
        { ops: ["+", "-"], prec: PREC.ADD, assoc: "left" },
        { ops: ["*", "/", "%"], prec: PREC.MULTIPLY, assoc: "left" },
        { ops: ["..."], prec: PREC.RANGE, assoc: "non" },
      ];
      return choice(
        ...BINOPS.map(({ ops, prec: level, assoc, rhs }) => {
          const op = ops.length === 1 ? ops[0] : choice(...ops);
          const rule = seq(
            field("left", $._Expr),
            field("op", op),
            field("right", rhs || $._Expr),
          );
          if (assoc === "right") return prec.right(level, rule);
          if (assoc === "left") return prec.left(level, rule);
          return prec(level, rule);
        }),
      );
    },
    EUnop: ($) =>
      choice(
        prec.right(
          PREC.UNARY,
          seq(
            field("op", choice("++", "--", "+", "-", "!", "~")),
            field("operand", $._expr_value),
          ),
        ),
        prec.left(
          PREC.POSTFIX,
          seq(field("operand", $._expr_lhs), field("op", choice("++", "--"))),
        ),
      ),
    EBlock: ($) => prec.dynamic(-1, seq("{", repeat($._expr_statement), "}")),
    EObjectDecl: ($) =>
      prec.dynamic(
        -1,
        seq(
          "{",
          // A trailing comma after the last field is legal Haxe, and is what
          // multi-line configuration objects are usually written with.
          optional(
            seq(
              commaSep1(
                seq(
                  field("name", choice($.identifier, $.String)),
                  ":",
                  field("value", $._Expr),
                ),
              ),
              optional(","),
            ),
          ),
          "}",
        ),
      ),
    EArrayDecl: ($) =>
      prec(
        1,
        seq(
          "[",
          optional(
            choice(
              // Haxe permits a trailing comma after the last element,
              // `["a", "b",]`, which hand-maintained tables rely on so that
              // every line can end the same way.
              seq(commaSep1($._Expr), optional(",")),
              alias($._comprehension_for, $.EFor),
              $.EWhile,
            ),
          ),
          "]",
        ),
      ),
    _comprehension_for: ($) =>
      seq(
        "for",
        "(",
        choice(
          seq(field("key", $.identifier), "=>", field("value", $.identifier)),
          field("var", $.identifier),
        ),
        "in",
        field("iterable", $._Expr),
        ")",
        field("body", $._comprehension_body),
      ),
    _comprehension_if: ($) =>
      seq(
        "if",
        "(",
        field("cond", $._Expr),
        ")",
        field("if", $._comprehension_body),
      ),
    _comprehension_body: ($) =>
      choice(
        alias($._comprehension_for, $.EFor),
        alias($._comprehension_if, $.EIf),
        $._Expr,
      ),
    EReturn: ($) => prec.right(PREC.CONTROL, seq("return", optional($._Expr))),
    EBreak: (_) => prec.right(PREC.CONTROL, "break"),
    EContinue: (_) => prec.right(PREC.CONTROL, "continue"),
    EThrow: ($) =>
      prec.right(PREC.CONTROL, seq("throw", field("expr", $._Expr))),
    ECast: ($) =>
      prec.right(
        PREC.CALL + 1,
        seq(
          "cast",
          choice(
            field("expr", $._Expr),
            seq(
              "(",
              field("expr", $._Expr),
              ",",
              field("type", $.ComplexType),
              ")",
            ),
          ),
        ),
      ),
    EUntyped: ($) => prec.right(seq("untyped", $._Expr)),
    EFor: ($) =>
      prec(
        PREC.PRIMARY + 1,
        seq(
          "for",
          "(",
          choice(
            seq(field("key", $.identifier), "=>", field("value", $.identifier)),
            field("var", $.identifier),
          ),
          "in",
          field("iterable", $._Expr),
          ")",
          field("body", $._block_or_expr),
        ),
      ),
    EIf: ($) =>
      prec.right(
        PREC.CONTROL + 1,
        seq(
          "if",
          "(",
          field("cond", $._Expr),
          ")",
          field("if", $._block_or_expr),
          optional(seq("else", field("else", $._block_or_expr))),
        ),
      ),
    EWhile: ($) =>
      prec.right(
        PREC.CONTROL + 1,
        choice(
          seq(
            "while",
            "(",
            field("cond", $._Expr),
            ")",
            field("body", $._block_or_expr),
          ),
          seq(
            "do",
            field("body", $._block_or_expr),
            "while",
            "(",
            field("cond", $._Expr),
            ")",
          ),
        ),
      ),
    ESwitch: ($) =>
      prec(
        PREC.CONTROL,
        seq(
          "switch",
          field("subject", $._Expr),
          "{",
          optional(field("cases", $.cases)),
          optional(field("default", $.switch_default)),
          "}",
        ),
      ),
    cases: ($) => repeat1($.switch_case),
    switch_case: ($) =>
      prec.dynamic(
        -1,
        seq(
          "case",
          field("patterns", commaSep1($._Expr)),
          optional(seq("if", "(", field("guard", $._Expr), ")")),
          ":",
          field("body", repeat(seq($._Expr, optional($._semicolon)))),
        ),
      ),
    switch_default: ($) =>
      prec.right(
        1,
        seq(
          choice("default", seq("case", alias($.wildcard_pattern, ""))),
          ":",
          field("body", repeat(seq($._Expr, optional($._semicolon)))),
        ),
      ),
    ETry: ($) =>
      prec.right(
        seq(
          "try",
          $._block_or_expr,
          repeat1(
            seq(
              "catch",
              "(",
              $.identifier,
              optional($._type_annotation),
              ")",
              field("body", $._block_or_expr),
            ),
          ),
        ),
      ),
    ECheckType: ($) =>
      prec.right(
        PREC.TYPE_ANNOTATION,
        seq("(", field("expr", $._Expr), $._type_annotation, ")"),
      ),
    EMeta: ($) =>
      prec.right(
        PREC.PRIMARY,
        seq(repeat1($.MetaDataEntry), field("expr", $._Expr)),
        // seq(
        //   repeat1($.MetaDataEntry),
        //   field("expr", choice($._type_decl, $.EFunction, $.EVars)),
        // ),
      ),

    macro: ($) => prec(PREC.MACRO, seq("macro", $._Expr)),
    reification: ($) =>
      prec(
        PREC.UNARY,
        seq(
          "$",
          choice(
            token.immediate(/[a-zA-Z_][a-zA-Z0-9_]*/), // identifier
            seq(token.immediate("{"), $._Expr, "}"),
            seq(token.immediate("e{"), $._Expr, "}"),
            seq(token.immediate("a{"), commaSep($._Expr), "}"),
            seq(
              token.immediate("b{"),
              repeat(seq($._Expr, optional($._semicolon))),
              "}",
            ),
            seq(token.immediate("i{"), $.identifier, "}"),
            seq(token.immediate("p{"), commaSep($.identifier), "}"),
            seq(token.immediate("v{"), $._Expr, "}"),
          ),
        ),
      ),

    type_trace: ($) =>
      prec(PREC.CALL + 1, seq("$type", "(", field("type", $._Expr), ")")),

    // ------------------------------------------------------------------------

    TypePath: ($) =>
      seq(
        repeat(seq(field("pack", $.package_name), ".")),
        field("name", $.type_name),
        optional(field("sub", prec.left(PREC.PRIMARY, seq(".", $.identifier)))),
        optional(field("params", $._type_arguments)),
      ),

    ComplexType: ($) =>
      choice(
        $._base_type,
        prec.left(
          2,
          field("TIntersection", seq($.ComplexType, "&", $.ComplexType)),
        ),
        prec.right(
          1,
          field(
            "TFunction",
            seq(
              // The `?` marks an optional argument, `MBitmap->?Object->Void`.
              // It is a separate alternative rather than an optional("?") on
              // the existing one, because without the "?" the two sequences
              // are identical and the ambiguity cannot be resolved.
              field("arg", choice($._base_type, $._optional_function_type_arg)),
              "->",
              field("ret", $.ComplexType),
            ),
          ),
        ),
      ),

    _optional_function_type_arg: ($) => seq("?", $._base_type),

    _base_type: ($) =>
      choice(
        //field("TPath", prec(PREC.PRIMARY, $.TypePath)),
        prec(PREC.PRIMARY, $.TypePath),
        $.TAnonymous,
        seq("(", $.ComplexType, ")"),
      ),

    TAnonymous: ($) =>
      prec.right(
        PREC.OBJECT_DECL,
        seq(
          "{",
          optional(repeat1(seq(">", field("extends", $.TypePath), ","))),
          choice(seq(commaSep($.Field), optional(",")), repeat($._class_field)),
          "}",
          optional($._semicolon),
        ),
      ),
    Field: ($) =>
      seq(
        optional($.optional),
        field("name", $.identifier),
        ":",
        field("type", $.ComplexType),
      ),

    TypeParameter: ($) =>
      seq(
        field("name", $.type_name),
        choice(
          optional(seq(":", field("constraint", $.ComplexType))),
          optional(seq("=", field("default", $.ComplexType))),
        ),
      ),

    MetaDataEntry: ($) =>
      prec.right(
        seq(
          "@",
          optional(token.immediate(":")),
          field(
            "name",
            choice(
              $.identifier,
              ...RESERVED_KEYWORDS.map((kw) => alias(kw, $.identifier)),
            ),
          ),
          optional(seq("(", field("params", commaSep($._Expr)), ")")),
        ),
      ),

    FunctionArg: ($) =>
      //ISSUE: without dynamic ECast fails
      prec.dynamic(
        1,
        seq(
          repeat($.MetaDataEntry),
          optional($.optional),
          optional(field("rest", $.rest)),
          field("name", $.identifier),
          optional(field("type", $._type_annotation)),
          optional(seq("=", field("value", $._Expr))),
        ),
      ),

    // ------------------------------------------------------------------------

    _type_decl: ($) =>
      seq(
        field("meta", repeat($.MetaDataEntry)),
        choice($.AbstractType, $.ClassType, $.DefType, $.EnumType),
      ),

    AbstractType: ($) =>
      seq(
        optional(repeat1(choice($.visibility, "enum"))),
        "abstract",
        $._type_decl_signature,
        "(",
        field("type", $.ComplexType),
        ")",
        repeat(
          choice(
            field("from", seq("from", $.TypePath)),
            field("to", seq("to", $.TypePath)),
          ),
        ),
        "{",
        repeat($._class_field),
        "}",
      ),

    ClassType: ($) =>
      seq(
        optional(repeat1(choice($.visibility, "abstract", "extern", "final"))),
        field("kind", choice("class", "interface")),
        $._type_decl_signature,
        // One interleaved repeat, not `extends` followed by `implements`: Haxe
        // accepts the clauses in either order, and
        // `class C implements I<Int> extends B<Int>` is legal. Written
        // sequentially, that order put the implements clause inside an ERROR
        // node -- the field still existed but was parented to the ERROR rather
        // than to ClassType, so a consumer reading ClassType's fields silently
        // saw a class with no interfaces. An interface may also extend several
        // others, which the repeat covers.
        repeat(
          choice(
            seq("extends", field("extends", $.TypePath)),
            seq("implements", field("implements", $.TypePath)),
          ),
        ),
        "{",
        repeat($._class_field),
        "}",
      ),
    _class_field: ($) =>
      seq(repeat($.MetaDataEntry), choice($.ClassVar, $.ClassMethod)),
    ClassVar: ($) =>
      prec.left(
        PREC.ASSIGN,
        seq(
          optional(
            repeat1(
              choice($.visibility, "dynamic", "extern", "inline", "static"),
            ),
          ),
          choice("var", "final"),
          field("name", $.identifier),
          optional($.property_accessor),
          optional($._type_annotation),
          optional(seq("=", $._Expr)),
          $._semicolon,
        ),
      ),
    property_accessor: ($) =>
      seq(
        "(",
        field("get", $.property_access),
        ",",
        field("set", $.property_access),
        ")",
      ),
    property_access: ($) =>
      choice(
        alias("default", $.default),
        alias("get", $.get),
        alias("set", $.set),
        alias("dynamic", $.dynamic),
        alias("never", $.never),
        $.null,
        $.identifier,
      ),
    ClassMethod: ($) =>
      prec.right(
        PREC.ASSIGN,
        seq(
          optional(
            repeat1(
              choice(
                $.visibility,
                "macro",
                "dynamic",
                "extern",
                "inline",
                "override",
                "final",
                "static",
              ),
            ),
          ),
          $._function_decl,
          optional($._semicolon),
        ),
      ),
    DefType: ($) =>
      seq(
        optional(repeat1(choice($.visibility, "extern"))),
        "typedef",
        $._type_decl_signature,
        "=",
        field("type", $.ComplexType),
        optional($._semicolon),
      ),

    EnumType: ($) =>
      seq(
        optional(repeat1(choice($.visibility, "extern"))),
        "enum",
        $._type_decl_signature,
        "{",
        repeat($.EnumConstructor),
        "}",
      ),
    EnumConstructor: ($) =>
      seq(
        field("meta", repeat($.MetaDataEntry)),
        field("name", $.identifier),
        optional(field("params", $._type_params)),
        optional(field("args", $._function_args)),
        $._semicolon,
      ),

    _function_decl: ($) =>
      seq(
        "function",
        optional(
          seq(
            field("name", choice($.identifier, alias("new", $.identifier))),
            optional(field("params", $._type_params)),
          ),
        ),
        $._function_args,
        optional(field("ret", $._type_annotation)),
        choice(field("body", $._block_or_expr), $._semicolon),
      ),

    _type_decl_signature: ($) =>
      seq(field("name", $.type_name), optional($._type_params)),

    // ------------------------------------------------------------------------

    Int: (_) =>
      choice(
        /0x[a-fA-F\d][a-fA-F\d_]*/,
        /0b[01][01_]*/,
        /0o[0-7][0-7_]*/,
        /\d[\d_]*/,
      ),

    // Three legal Float spellings that did not parse:
    //   - a leading-dot fraction, `.5`, as in `lineStyle(.5, color)`.
    //   - an uppercase exponent, `1.0E5`. The marker was written `[ee]`, a
    //     character class holding the same letter twice, so only lowercase
    //     `e` was ever accepted.
    //   - a single digit before a bare exponent, `1e5`. The form required
    //     `\d[\d_]+`, two or more digits, so `10e5` parsed and `1e5` did not.
    Float: (_) =>
      choice(
        /\d[\d_]*\.\d[\d_]*([eE][+-]?\d[\d_]*)?/,
        /\.\d[\d_]*([eE][+-]?\d[\d_]*)?/,
        /\d[\d_]*[eE][+-]?\d[\d_]*/,
      ),

    String: ($) =>
      choice(
        seq(
          "'",
          repeat(
            choice(
              alias(token.immediate(prec(1, /[^'\\$]+/)), $.fragment),
              $.escape_sequence,
              $.interpolation,
            ),
          ),
          "'",
        ),
        seq(
          '"',
          repeat(
            choice(
              alias(token.immediate(prec(1, /[^"\\]+/)), $.fragment),
              $.escape_sequence,
            ),
          ),
          '"',
        ),
      ),
    escape_sequence: () => token.immediate(seq("\\", /./)),
    interpolation: ($) =>
      seq(
        "$",
        choice(
          seq(token.immediate("{"), $._Expr, "}"),
          alias(token.immediate(/[a-zA-Z_][a-zA-Z0-9_]*/), $.identifier),
        ),
      ),

    Regexp: (_) =>
      seq("~/", repeat(choice(/[^/\\\n]/, /\\./)), "/", /[gimsu]*/),

    true: (_) => "true",
    false: (_) => "false",
    null: (_) => "null",
    this: (_) => "this",
    super: (_) => "super",
    rest: (_) => "...",

    // ------------------------------------------------------------------------

    _dot_path: ($) => dotSep1($.package_name),

    _type_annotation: ($) =>
      prec(PREC.TYPE_ANNOTATION, seq(":", field("type", $.ComplexType))),

    _type_params: ($) => seq("<", commaSep1($.TypeParameter), ">"),
    _type_arguments: ($) => seq("<", commaSep($.ComplexType), ">"),

    _function_args: ($) =>
      seq("(", field("args", commaSep($.FunctionArg)), ")"),

    // ------------------------------------------------------------------------

    visibility: (_) => choice("public", "private"),

    identifier: (_) => /[a-zA-Z_][a-zA-Z0-9_]*/,
    package_name: (_) => /[a-z_][a-zA-Z0-9_]*/,
    type_name: (_) => /[A-Z][a-zA-Z0-9_]*/,

    optional: (_) => "?",
    wildcard: (_) => "*",
    wildcard_pattern: (_) => "_",
    _semicolon: (_) => ";",

    comment: ($) => choice($.line_comment, $.block_comment),
    line_comment: (_) => token(seq("//", /[^\n]*/)),
    block_comment: (_) => token(seq("/*", /[^*]*\*+([^/*][^*]*\*+)*\//)),

    // ------------------------------------------------------------------------

    conditional: ($) =>
      prec.right(
        PREC.CONDITIONAL,
        seq(
          "#if",
          $.compile_condition,
          repeat($._conditional_body),
          repeat($.conditional_elseif),
          optional($.conditional_else),
          $.conditional_end,
        ),
      ),
    compile_condition: ($) =>
      choice(
        $.identifier,
        alias("macro", $.identifier),
        $.Int,
        $.Float,
        $.String,
        seq("(", $.compile_condition, ")"),
        prec.left(7, seq($.compile_condition, ".", $.identifier)),
        prec.right(6, seq("!", $.compile_condition)),
        prec.left(
          5,
          seq($.compile_condition, choice("*", "/", "%"), $.compile_condition),
        ),
        prec.left(
          4,
          seq($.compile_condition, choice("+", "-"), $.compile_condition),
        ),
        prec.left(
          3,
          seq(
            $.compile_condition,
            choice(">", ">=", "<", "<=", "==", "!="),
            $.compile_condition,
          ),
        ),
        prec.left(2, seq($.compile_condition, "&&", $.compile_condition)),
        prec.left(1, seq($.compile_condition, "||", $.compile_condition)),
      ),
    _conditional_body: ($) =>
      choice(
        // ISSUE: adding everything sucks
        $.import,
        $.using,
        $._class_field,
        $._expr_statement,
        $._type_decl,
        $.visibility,
        $.conditional_error,
        repeat1($.MetaDataEntry),
      ),
    conditional_elseif: ($) =>
      seq("#elseif", $.compile_condition, repeat($._conditional_body)),
    conditional_else: ($) => seq("#else", repeat($._conditional_body)),
    conditional_error: ($) => seq("#error", $.String),
    conditional_end: (_) => "#end",
  },
});
