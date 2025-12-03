module.exports = {
  primitive: $ => choice(
    $.number,
    $.boolean
  ),
  number: $ => {
    const hexLiteral = seq(
      choice('0x', '0X'),
      /[\da-fA-F](_?[\da-fA-F])*/
    )

    const decimalDigits = /\d(_?\d)*/
    const exponentPart = seq(choice('e', 'E'), optional(choice('-', '+')), decimalDigits)

    const binaryLiteral = seq(choice('0b', '0B'), /[0-1](_?[0-1])*/)

    const octalLiteral = seq('0', /[0-7](_?[0-7])*/)

    const decimalIntegerLiteral = choice(
      '0',
      seq(/[1-9]/, optional(seq(optional('_'), decimalDigits)))
    )

    const decimalLiteral = choice(
      // 111.555e99
      seq(decimalIntegerLiteral, '.', optional(decimalDigits), optional(exponentPart)),
      // .555e99
      seq('.', decimalDigits, optional(exponentPart)),
      // 111e99
      seq(decimalIntegerLiteral, exponentPart),
      // 111
      decimalDigits
    )

    return token(choice(
      decimalLiteral,
      hexLiteral,
      binaryLiteral,
      octalLiteral
    ))
  },
  boolean: $ => choice('true', 'false')
}
