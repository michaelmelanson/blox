/**
 * @file A parser for the Blox language
 * @author Michael Melanson <michael@michaelmelanson.net>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "blox",

  rules: {
    source_file: ($) => repeat(field("statement", $._statement)),
    comment: ($) => choice(token(seq("#", /.*/))),

    block: ($) => seq("{", repeat(field("statement", $._statement)), "}"),
    _statement: ($) =>
      choice($.definition, $.binding, $.import, $.expression_statement),
    binding: ($) =>
      seq(
        "let",
        field("name", $.identifier),
        "=",
        field("value", $._expression),
      ),
    definition: ($) =>
      seq(
        "def",
        field("name", $.identifier),
        "(",
        optional($._function_parameters),
        ")",
        field("body", $.block),
      ),
    _function_parameters: ($) =>
      seq(
        field("parameter", $.identifier),
        repeat(seq(",", field("parameter", $.identifier))),
      ),
    import: ($) =>
      seq(
        "import",
        "{",
        $.imported_symbol,
        repeat(seq(",", $.imported_symbol)),
        "}",
        "from",
        field("path", $.string),
      ),
    imported_symbol: ($) =>
      seq(
        field("identifier", $.identifier),
        optional(seq("as", field("alias", $.identifier))),
      ),
    expression_statement: ($) => field("expression", $._expression),
    _expression: ($) => choice($.term, $.unary_expression, $.binary_expression),
    term: ($) =>
      choice(
        $.if_expression,
        $.array_index,
        $.object_index,
        $.function_call,
        $._value,
        $.group_term,
      ),
    group_term: ($) => seq("(", field("expression", $._expression), ")"),
    unary_expression: ($) =>
      prec.left(
        1,
        choice(prec(3, seq($.negate, $.term)), prec(3, seq($.not, $.term))),
      ),
    binary_expression: ($) => {
      let operators = [
        { operator: $.multiply, precedence: 2 },
        { operator: $.divide, precedence: 2 },
        { operator: $.concatenate, precedence: 1 },
        { operator: $.add, precedence: 1 },
        { operator: $.subtract, precedence: 1 },
        { operator: $.multiply, precedence: 1 },
        { operator: $.divide, precedence: 1 },
        { operator: $.equal, precedence: 1 },
        { operator: $.not_equal, precedence: 1 },
        { operator: $.less_than, precedence: 1 },
        { operator: $.less_or_equal, precedence: 1 },
        { operator: $.greater_than, precedence: 1 },
        { operator: $.greater_or_equal, precedence: 1 },
      ];

      return prec.right(
        choice(
          ...operators.map((op) =>
            prec.left(
              op.precedence,
              seq(
                field("lhs", $._expression),
                field("operator", op.operator),
                field("rhs", $._expression),
              ),
            ),
          ),
        ),
      );
    },
    _value: ($) => choice($.literal, $.identifier, $.array, $.object),
    if_expression: ($) =>
      seq(
        "if",
        field("condition", $._expression),
        field("body", $.block),
        repeat(field("elseif", $.elseif_expression)),
        field("else", optional($.else_expression)),
      ),
    elseif_expression: ($) =>
      seq(
        "else",
        "if",
        field("condition", $._expression),
        field("body", $.block),
      ),
    else_expression: ($) => seq("else", field("body", $.block)),
    function_call: ($) =>
      prec(
        1,
        seq(field("name", $.identifier), "(", optional($._argument_list), ")"),
      ),
    _argument_list: ($) =>
      seq(
        field("argument", $.argument),
        repeat(seq(",", field("argument", $.argument))),
      ),
    argument: ($) =>
      seq(field("name", $.identifier), ":", field("value", $._expression)),

    array: ($) => seq("[", optional($._array_members), "]"),
    _array_members: ($) =>
      seq(
        field("member", $._expression),
        repeat(seq(",", field("member", $._expression))),
      ),
    array_index: ($) =>
      prec(
        3,
        seq(
          field("base", $._expression),
          "[",
          field("index", $._expression),
          "]",
        ),
      ),

    object: ($) => seq("{", optional($._object_members), "}"),
    _object_members: ($) =>
      seq(
        field("member", $.object_member),
        repeat(seq(",", field("member", $.object_member))),
      ),
    object_member: ($) =>
      seq(field("key", $.identifier), ":", field("value", $._expression)),
    object_index: ($) =>
      prec(
        4,
        seq(field("base", $._expression), ".", field("index", $.identifier)),
      ),

    unary_operator: ($) => choice($.negate, $.not),
    not: ($) => "!",
    negate: ($) => "-",
    multiply: ($) => "*",
    divide: ($) => "/",
    concatenate: ($) => "++",
    add: ($) => "+",
    subtract: ($) => "-",
    equal: ($) => "==",
    not_equal: ($) => "!=",
    greater_or_equal: ($) => ">=",
    greater_than: ($) => ">",
    less_or_equal: ($) => "<=",
    less_than: ($) => "<",

    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    literal: ($) => choice($.boolean, $.number, $.string, $.symbol),
    boolean: ($) => choice($.boolean_true, $.boolean_false),
    boolean_true: ($) => "true",
    boolean_false: ($) => "false",
    number: ($) => /-?[0-9]+(\.[0-9]+)?/,
    string: ($) =>
      choice(
        seq('"', repeat(choice(/[^"]/, '\\"')), '"'),
        seq("'", repeat(choice(/[^']/, "\\'")), "'"),
      ),
    symbol: ($) => seq(":", /[a-zA-Z]+/),
  },

  extras: ($) => [/\s/, $.comment],
});
