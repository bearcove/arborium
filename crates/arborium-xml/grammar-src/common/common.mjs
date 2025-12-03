/**
 * @file Reusable constructs
 * @author ObserverOfTime
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

/**
 * @param {"'" | ''} q
 * @see {@link https://www.w3.org/TR/xml/#NT-PubidChar}
 */
export const pubid_char = (q) =>
  new RegExp(`[ \\r\\na-zA-Z0-9\\-${q}()+,./:=?;!*#@$_%]*`);

/**
 * @param {GrammarSymbols<any>} $
 * @param {'"' | "'"} q
 */
export const att_value = ($, q) =>
  seq(
    q,
    field(
      'content',
      repeat(choice(
        new RegExp(`[^<&${q}]`),
        $._Reference
      ))
    ),
    q
  );

/**
 * @param {GrammarSymbols<any>} $
 * @param {'"' | "'"} q
 */
export const entity_value = ($, q) =>
  seq(
    q,
    field(
      'content',
      repeat(choice(
        new RegExp(`[^<%&${q}]`),
        $.PEReference,
        $._Reference
      ))
    ),
    q
  );


/** @param {RuleOrLiteral[]} choices */
export const str = (...choices) =>
  choice(
    seq("'", ...choices, "'"),
    seq('"', ...choices, '"')
  );

/**
 * @param {GrammarSymbols<any>} $
 * @param {RuleOrLiteral[]} choices
 */
export const ref = ($, ...choices) =>
  choice(...choices, $.PEReference);

/** @param {RuleOrLiteral[]} rules */
export const rseq = (...rules) => repeat(seq(...rules));

/** @param {RuleOrLiteral[]} rules */
export const rseq1 = (...rules) => repeat1(seq(...rules));

const O = optional;

/** @type {Record<string, ($: GrammarSymbols<any>) => RuleOrLiteral>} */
export const rules = {
    _markupdecl: $ => choice(
      $.elementdecl,
      $.AttlistDecl,
      $._EntityDecl,
      $.NotationDecl,
      $.PI,
      $.Comment
    ),

    _DeclSep: $ => choice($.PEReference, $._S),

    elementdecl: $ => seq(
      '<!',
      'ELEMENT',
      $._S,
      ref($, $.Name),
      $._S,
      $.contentspec,
      O($._S),
      '>'
    ),

    contentspec: $ => choice(
      'EMPTY',
      'ANY',
      $.Mixed,
      $.children,
      $.PEReference
    ),

    Mixed: $ => choice(
      seq(
        '(',
        O($._S),
        ref($, '#PCDATA'),
        rseq(
          O($._S),
          '|',
          O($._S),
          ref($, $.Name),
        ),
        O($._S),
        rseq(
          $.PEReference,
          O($._S)
        ),
        ')',
        '*'
      ),
      prec(-1, seq(
        '(',
        O($._S),
        ref($, '#PCDATA'),
        O($._S),
        rseq(
          $.PEReference,
          O($._S)
        ),
        ')'
      ))
    ),

    children: $ => prec(1, seq(
      $._choice,
      O(choice('?', '*', '+'))
    )),

    _cp: $ => prec.left(seq(
      ref($, $.Name, $._choice),
      O(choice('?', '*', '+'))
    )),

    _choice: $ => seq(
      '(',
      O($._S),
      $._cp,
      rseq(
        O($._S),
        choice('|', ','),
        O($._S),
        $._cp
      ),
      rseq(
        O($._S),
        $.PEReference,
      ),
      O($._S),
      ')'
    ),

    AttlistDecl: $ => seq(
      '<!',
      'ATTLIST',
      $._S,
      ref($, $.Name),
      repeat(choice(
        $.AttDef,
        seq($._S, $.PEReference)
      )),
      O($._S),
      '>'
    ),

    AttDef: $ => prec.right(seq(
      $._S,
      ref($, $.Name),
      $._S,
      $._AttType,
      O(seq($._S, $.DefaultDecl))
    )),

    _AttType: $ => choice(
      $.StringType,
      $.TokenizedType,
      $._EnumeratedType,
      $.PEReference
    ),

    StringType: _ => 'CDATA',

    TokenizedType: _ => token(choice(
      'ID',
      'IDREF',
      'IDREFS',
      'ENTITY',
      'ENTITIES',
      'NMTOKEN',
      'NMTOKENS',
    )),

    _EnumeratedType: $ => choice(
      $.NotationType,
      $.Enumeration
    ),

    NotationType: $ => seq(
      'NOTATION',
      $._S,
      '(',
      O($._S),
      ref($, $.Name),
      rseq(
        O($._S),
        '|',
        O($._S)
      ),
      ref($, $.Name),
      O($._S),
      ')'
    ),

    Enumeration: $ => seq(
      '(',
      O($._S),
      $.Nmtoken,
      rseq(
        O($._S),
        '|',
        O($._S),
        $.Nmtoken
      ),
      O($._S),
      ')'
    ),

    DefaultDecl: $ => choice(
      '#REQUIRED',
      '#IMPLIED',
      seq(
        O(seq('#FIXED', $._S)),
        $.AttValue
      ),
      $.PEReference
    ),

    _EntityDecl: $ => choice(
      $.GEDecl,
      $.PEDecl
    ),

    GEDecl: $ => seq(
      '<!',
      'ENTITY',
      $._S,
      ref($, $.Name),
      $._S,
      choice(
        $.EntityValue,
        seq(
          $.ExternalID,
          O($.NDataDecl)
        )
      ),
      O($._S),
      '>'
    ),

    PEDecl: $ => seq(
      '<!',
      'ENTITY',
      $._S,
      '%',
      $._S,
      $.Name,
      $._S,
      choice(
        $.EntityValue,
        $.ExternalID
      ),
      O($._S),
      '>'
    ),

    EntityValue: $ => choice(
      entity_value($, '"'),
      entity_value($, "'")
    ),

    NDataDecl: $ => seq($._S, 'NDATA', $._S, ref($, $.Name)),

    NotationDecl: $ => seq(
      '<!',
      'NOTATION',
      $._S,
      ref($, $.Name),
      $._S,
      choice($.ExternalID, $.PublicID),
      O($._S),
      '>'
    ),

    PEReference: $ => seq('%', $.Name, ';'),

    _S: _ => /[ \t\r\n]+/,

    Name: _ => /[a-zA-Z_][a-zA-Z0-9_:.·-]*/,

    Nmtoken: _ => /[a-zA-Z0-9_:.·-]+/,

    _Reference: $ => choice($.EntityRef, $.CharRef),

    EntityRef: $ => seq('&', $.Name, ';'),

    CharRef: _ => choice(
      seq('&#', /[0-9]+/, ';'),
      seq('&#x', /[0-9a-fA-F]+/, ';')
    ),

    AttValue: $ => choice(
      att_value($, '"'),
      att_value($, "'")
    ),

    ExternalID: $ => choice(
      seq('SYSTEM', $._S, $.SystemLiteral),
      seq('PUBLIC', $._S, $.PubidLiteral, $._S, $.SystemLiteral)
    ),

    PublicID: $ => prec.right(
      seq(ref($, 'PUBLIC'), $._S, $.PubidLiteral)
    ),

    SystemLiteral: $ => choice(
      seq('"', alias(/[^"]*/, $.URI), '"'),
      seq("'", alias(/[^']*/, $.URI), "'")
    ),

    PubidLiteral: _ => choice(
      seq('"', pubid_char("'"), '"'),
      seq("'", pubid_char(''), "'")
    ),

    _VersionInfo: $ => seq(
      $._S,
      'version',
      $._Eq,
      str($.VersionNum)
    ),

    VersionNum: _ => /1\.[0-9]+/,

    _EncodingDecl: $ => seq(
      $._S,
      'encoding',
      $._Eq,
      str($.EncName)
    ),

    EncName: _ => /[A-Za-z][A-Za-z0-9._\-]*/,

    PI: $ => seq(
      '<?',
      $.PITarget,
      O(seq($._S, $._pi_content)),
      '?>'
    ),

    _Eq: $ => seq(O($._S), '=', O($._S))
}
