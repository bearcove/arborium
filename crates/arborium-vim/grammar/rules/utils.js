function key_val_arg(arg, ...args) {
  if (args.length > 0) {
    return seq(field("name", arg), token.immediate("="), field("val", ...args));
  } else return field("name", arg);
}

function maybe_bang($, cmd_name) {
  return seq(cmd_name, optional($.bang));
}

function _to_case_insensitive(a) {
  var ca = a.charCodeAt(0);
  if (ca >= 97 && ca <= 122) return `[${a}${a.toUpperCase()}]`;
  if (ca >= 65 && ca <= 90) return `[${a.toLowerCase()}${a}]`;
  return a;
}

function case_insensitive(keyword) {
  return new RegExp(keyword.split("").map(_to_case_insensitive).join(""));
}

function keyword(gram, name) {
  return alias(gram["_" + name], name);
}

function sub_cmd(sub, ...args) {
  if (args.length > 0) {
    return seq(field("sub", sub), ...args);
  } else {
    return field("sub", sub);
  }
}

function commaSep(rule) {
  return optional(commaSep1(rule));
}

function commaSep1(rule) {
  return sep1(rule, ",");
}

function sep1(rule, separator) {
  return seq(rule, repeat(seq(separator, rule)));
}

function command($, cmd, ...args) {
  return seq(keyword($, cmd), ...args);
}

function bang_command($, cmd, ...args) {
  return seq(maybe_bang($, keyword($, cmd)), ...args);
}

module.exports = {
  key_val_arg,
  maybe_bang,
  keyword,
  sub_cmd,
  commaSep,
  commaSep1,
  sep1,
  command,
  bang_command,
  case_insensitive,
};
