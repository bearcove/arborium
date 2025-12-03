/**
 * Creates a rule to match one or more of the rules separated by a comma
 *
 * @param {Rule} rule
 *
 * @return {SeqRule}
 *
 */
function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}

/**
 * Creates a rule to optionally match one or more of the rules separated by a comma
 *
 * @param {Rule} rule
 *
 * @return {ChoiceRule}
 *
 */
function commaSep(rule) {
  return optional(commaSep1(rule));
}

/**
 * @param {Rule} rule
 * @param {Rule|string} sep
 * @return {SeqRule}
 */
function anySep1(rule, sep) {
  return seq(rule, repeat(seq(sep, rule)));
}

/**
 * @param {Rule} rule
 * @param {Rule} sep
 * @return {ChoiceRule}
 */
function anySep(rule, sep) {
  return optional(anySep1(rule, sep));
}

function escaped_ch(ch, exclude_newline, ...args) {
  let escaped = ch;
  const mapped = ch.split('').map(v => {
    if (v === ']') {
      return '\\]';
    }
    if (v === '[') {
      return '\\[';
    }
    return `\\${v}`;
  });

  if (ch === ']') {
    escaped = '\\]';
  }
  if (ch === '[') {
    escaped = '\\[';
  }
  if (exclude_newline) {
    escaped += '\r\n';
  }

  return choice(new RegExp(`[^${escaped}]`), ...mapped, ...args);
}

exports.commaSep1 = commaSep1;
exports.commaSep = commaSep;
exports.anySep1 = anySep1;
exports.anySep = anySep;
exports.escaped_ch = escaped_ch;
