/* eslint-disable no-undef */
/// <reference types="tree-sitter-cli/dsl.d.ts" />

// ZenScript grammar for CraftTweaker, vendored from
//   ikexing-cn/tree-sitter-zenscript @ e31ceae2a298bc9d38e013e0416280f0cb386351
// and completed using the official CraftTweaker/ZenScript Java parser as the
// reference for operators, precedence, and statement grammar.
//
// Expression precedence, top-down (lowest to highest):
//   1. assignment  (=, +=, -=, *=, /=, %=, |=, &=, ^=, ~=)
//   2. conditional (? :)
//   3. ||          (T_OR2)
//   4. &&          (T_AND2)
//   5. |  (bitwise or)
//   6. ^  (bitwise xor)
//   7. &  (bitwise and)
//   8. comparison (==, !=, <, <=, >, >=, in)
//   9. add  (+, -, ~)         <-- ~ is string concatenation
//  10. mul  (*, /, %)
//  11. unary (-, !)
//  12. postfix (.member, [index], (call), as Type, instanceof Type, .. / to)

module.exports = grammar({
  name: 'zenscript',

  extras: $ => [
    $.comment,
    /[\s\p{Zs}\uFEFF\u2060\u200B]/,
  ],

  word: $ => $.identifier,

  // The `static` and `global` keywords can show up either as a leading
  // modifier on a member or as the entire token of an expression. The
  // conflict list below tells tree-sitter to accept either parse.
  conflicts: $ => [
    [$._expression, $.variable_declaration],
    [$._expression, $.formal_parameter],
    [$._type_literal, $._expression],
    [$.postfix_expression, $._expression],
    [$._top_level_element, $._statement],
    [$.block_statement, $.map_literal],
    [$.block_statement, $.lambda_body],
    [$.function_body, $.lambda_body],
    [$.function_body, $.class_body],
    [$.range_expression, $.assignment_expression],
    [$.range_expression, $.index_expression],
    [$.range_expression, $.call_expression],
    [$.range_expression, $.type_cast_expression],
    [$.range_expression, $.instanceof_expression],
    [$.range_expression, $.member_access_expression],
    [$.range_expression, $.ternary_expression],
    [$.parenthesized_expression, $.argument_list],
    [$.array_literal, $.index_expression],
    [$.qualified_name, $._type_literal],
  ],

  rules: {
    // ========================================================================
    // Top level
    // ========================================================================

    compilation_unit: $ => repeat($._top_level_element),

    _top_level_element: $ =>
      choice(
        $.import_declaration,
        $.class_declaration,
        $.function_declaration,
        $.expand_function_declaration,
        $.version_statement,
        $._statement,
      ),

    // ========================================================================
    // Imports
    // ========================================================================

    import_declaration: $ => seq(
      'import',
      $._name,
      optional(seq($.as, field('alias', $.identifier))),
      ';',
    ),

    // ========================================================================
    // Classes (zenClass / frigginClass)
    // ========================================================================

    class_declaration: $ => seq(
      field('keyword', choice('zenClass', 'frigginClass')),
      field('name', $.identifier),
      field('body', $.class_body),
    ),

    class_body: $ => seq(
      '{',
      repeat($._class_member),
      '}',
    ),

    _class_member: $ => choice(
      $.variable_declaration,
      $.function_declaration,
      $.constructor_declaration,
      $.expression_statement,
    ),

    constructor_declaration: $ => seq(
      field('keyword', choice('zenConstructor', 'frigginConstructor')),
      field('parameters', $.formal_parameter_list),
      field('body', $.function_body),
    ),

    // ========================================================================
    // Functions
    // ========================================================================

    function_declaration: $ => seq(
      optional($.static),
      'function',
      field('name', $.identifier),
      field('parameters', $.formal_parameter_list),
      optional(seq($.as, field('return_type', $._type_literal))),
      field('body', $.function_body),
    ),

    expand_function_declaration: $ => seq(
      '$expand',
      field('type', $._type_literal),
      '$',
      field('name', $.identifier),
      field('parameters', $.formal_parameter_list),
      optional(seq($.as, field('return_type', $._type_literal))),
      field('body', $.function_body),
    ),

    function_body: $ => seq(
      '{',
      optional(repeat1($._statement)),
      '}',
    ),

    // ========================================================================
    // Statements
    // ========================================================================

    _statement: $ => choice(
      $.block_statement,
      $.return_statement,
      $.break_statement,
      $.continue_statement,
      $.if_statement,
      $.foreach_statement,
      $.while_statement,
      $.version_statement,
      $.variable_declaration,
      $.expression_statement,
    ),

    block_statement: $ => seq(
      '{',
      optional(repeat1($._statement)),
      '}',
    ),

    return_statement: $ => seq(
      'return',
      optional($._expression),
      ';',
    ),

    break_statement: $ => seq('break', ';'),

    continue_statement: $ => seq('continue', ';'),

    if_statement: $ => prec.left(seq(
      'if',
      field('condition', $._expression),
      field('consequence', $._statement),
      optional(seq('else', field('alternative', $._statement))),
    )),

    foreach_statement: $ => seq(
      'for',
      field('variables', $.foreach_variable_list),
      'in',
      field('iterable', $._expression),
      field('body', $._statement),
    ),

    foreach_variable_list: $ => seq(
      $.identifier,
      optional(repeat1(seq(',', $.identifier))),
    ),

    while_statement: $ => seq(
      'while',
      field('condition', $._expression),
      field('body', $._statement),
    ),

    version_statement: $ => seq('version', field('number', $.number_literal), ';'),

    variable_declaration: $ => seq(
      field('prefix', choice('var', 'val', $.static, $.global)),
      field('name', $.identifier),
      optional(seq($.as, field('type', $._type_literal))),
      optional(seq('=', field('initializer', $._expression))),
      ';',
    ),

    expression_statement: $ => seq(
      $._expression,
      ';',
    ),

    // ========================================================================
    // Expressions
    //
    // Built bottom-up: atoms first, then unary, then postfix, then binary
    // (with explicit precedence levels), then ternary, then assignment.
    // The `as` and `instanceof` operators are postfix in the official parser
    // (i.e. `value as Type`, not `value as Type value`).
    // ========================================================================

    _expression: $ => choice(
      $.primary_expression,
      $.unary_expression,
      $.postfix_expression,
      $.binary_expression,
      $.ternary_expression,
      $.assignment_expression,
    ),

    primary_expression: $ => choice(
      $.parenthesized_expression,
      $.identifier,
      $.string_literal,
      $.number_literal,
      $.boolean_literal,
      $.null_literal,
      $.array_literal,
      $.map_literal,
      $.bracket_handler,
      $.lambda_expression,
    ),

    parenthesized_expression: $ => seq('(', $._expression, ')'),

    string_literal: $ => choice(
      seq('"', repeat($.string_content), '"'),
      seq("'", repeat($.string_content), "'"),
    ),

    string_content: $ => choice(
      $.string_fragment,
      $.escape_sequence,
    ),

    string_fragment: $ => /[^\\"'$]+/,

    escape_sequence: $ => token(prec(1, /\\(?:[btnfr"'\\$]|u[0-9a-fA-F]{4})/)),

    number_literal: $ => token(choice(
      /0[xX][0-9a-fA-F]+[lL]?/,
      /-?(?:[0-9]+\.[0-9]+(?:[eE][+-]?[0-9]+)?[fFdD]?|[0-9]+[eE][+-]?[0-9]+[fFdD]?|[0-9]+[fFdD]?)/,
    )),

    boolean_literal: $ => choice('true', 'false'),

    null_literal: $ => 'null',

    array_literal: $ => seq(
      '[',
      optional(seq($._expression, repeat(seq(',', $._expression)))),
      ']',
    ),

    map_literal: $ => seq(
      '{',
      optional(seq($.map_entry, repeat(seq(',', $.map_entry)))),
      '}',
    ),

    map_entry: $ => seq(
      field('key', $._expression),
      ':',
      field('value', $._expression),
    ),

    bracket_handler: $ => seq(
      '<',
      $.bracket_handler_content,
      '>',
    ),

    bracket_handler_content: $ => /[^<>]+/,

    // ---- Lambdas ------------------------------------------------------------
    //
    // `function (params) as Type { body }` — the body is a plain block
    // statement, parsed as a series of statements until the matching `}`.

    lambda_expression: $ => seq(
      'function',
      field('parameters', $.formal_parameter_list),
      optional(seq('as', field('return_type', $._type_literal))),
      field('body', $.lambda_body),
    ),

    lambda_body: $ => seq(
      '{',
      optional(repeat1($._statement)),
      '}',
    ),

    // ---- Unary --------------------------------------------------------------

    unary_expression: $ => prec.right(seq(
      field('operator', choice('!', '-')),
      field('argument', $._expression),
    )),

    // ---- Postfix ------------------------------------------------------------
    //
    // Member access, indexing, calls, type cast, instanceof, and the range
    // operators all hang off the postfix position in the official parser.

    postfix_expression: $ => choice(
      $.call_expression,
      $.member_access_expression,
      $.index_expression,
      $.type_cast_expression,
      $.instanceof_expression,
      $.range_expression,
    ),

    call_expression: $ => prec(1, seq(
      field('function', $._expression),
      field('arguments', $.argument_list),
    )),

    argument_list: $ => seq(
      '(',
      optional(seq($._expression, repeat(seq(',', $._expression)))),
      ')',
    ),

    member_access_expression: $ => seq(
      field('object', $._expression),
      '.',
      field('property', $.identifier),
    ),

    index_expression: $ => seq(
      field('object', $._expression),
      '[',
      field('index', $._expression),
      ']',
    ),

    type_cast_expression: $ => seq(
      field('value', $._expression),
      'as',
      field('type', $._type_literal),
    ),

    instanceof_expression: $ => seq(
      field('value', $._expression),
      'instanceof',
      field('type', $._type_literal),
    ),

    range_expression: $ => seq(
      field('start', $._expression),
      field('operator', choice('..', alias('to', '..'))),
      field('end', $.primary_expression),
    ),

    // ---- Binary -------------------------------------------------------------
    //
    // Precedence climbs from low to high. Within a single level, `prec.left`
    // forces left-associativity, which matches how the language actually
    // parses `a - b - c` as `(a - b) - c`.

    binary_expression: $ => choice(
      prec.left(1,  seq($._expression, '||', $._expression)),
      prec.left(2,  seq($._expression, '&&', $._expression)),
      prec.left(3,  seq($._expression, '|',  $._expression)),
      prec.left(4,  seq($._expression, '^',  $._expression)),
      prec.left(5,  seq($._expression, '&',  $._expression)),
      prec.left(6,  seq($._expression, choice('==', '!='), $._expression)),
      prec.left(7,  seq($._expression, choice('<', '>', '<=', '>='), $._expression)),
      prec.left(7,  seq($._expression, 'has', $._expression)),
      prec.left(8,  seq($._expression, 'in', $._expression)),
      prec.left(9,  seq($._expression, choice('+', '-', '~'), $._expression)),
      prec.left(10, seq($._expression, choice('*', '/', '%'), $._expression)),
    ),

    // ---- Ternary ------------------------------------------------------------

    ternary_expression: $ => prec.right(1, seq(
      field('condition', $._expression),
      '?',
      field('consequence', $._expression),
      ':',
      field('alternative', $._expression),
    )),

    // ---- Assignment ---------------------------------------------------------

    assignment_expression: $ => prec.right(seq(
      field('left', $._expression),
      field('operator', choice(
        '=',
        '+=', '-=', '*=', '/=', '%=',
        '~=',
        '|=', '&=', '^=',
      )),
      field('right', $._expression),
    )),

    // ========================================================================
    // Names and identifiers
    // ========================================================================

    _name: $ => choice(
      $.identifier,
      $.qualified_name,
    ),

    qualified_name: $ => seq(
      field('scope', $._name),
      '.',
      field('name', $.identifier),
    ),

    identifier: $ => /[a-zA-Z_][a-zA-Z_0-9]*/,

    // ========================================================================
    // Function parameters
    // ========================================================================

    formal_parameter_list: $ => seq(
      '(',
      optional(seq($.formal_parameter, repeat(seq(',', $.formal_parameter)))),
      ')',
    ),

    formal_parameter: $ => seq(
      field('name', $.identifier),
      optional(seq($.as, field('type', $._type_literal))),
      optional(seq('=', field('default', $._expression))),
    ),

    // ========================================================================
    // Types
    // ========================================================================

    _type_literal: $ => choice(
      alias($._name, $.class_type),
      $.function_type,
      $.list_type,
      $.array_type,
      $.map_type,
      $.primitive_type,
    ),

    type_literal_list: $ => seq(
      $._type_literal,
      optional(repeat1(seq(',', $._type_literal))),
    ),

    function_type: $ => seq(
      'function',
      '(',
      $.type_literal_list,
      ')',
      field('return_type', $._type_literal),
    ),

    list_type: $ => seq(
      '[',
      $._type_literal,
      ']',
    ),

    array_type: $ => prec(1, seq(
      $._type_literal,
      '[',
      ']',
    )),

    map_type: $ => prec(1, seq(
      field('value', $._type_literal),
      '[',
      field('key', $._type_literal),
      ']',
    )),

    primitive_type: $ => choice(
      'any', 'byte', 'short', 'int', 'long', 'float', 'double', 'bool', 'void', 'string',
    ),

    // ========================================================================
    // Comments
    //
    // `#` lines that look like preprocessor directives (`#priority`, `#sideonly`,
    // `#modloaded`, etc.) are deliberately NOT handled here — the upstream
    // grammar did not support them, and tree-sitter can't disambiguate
    // `#priority 100` (preprocessor) from `# this is a comment` without an
    // external scanner. Such lines are matched as comments.
    // ========================================================================

    comment: $ => token(
      choice(
        seq('//', /.*/),
        seq('#', /.*/),
        seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'),
      ),
    ),

    // ========================================================================
    // Keyword tokens
    // ========================================================================

    as: $ => token('as'),
    static: $ => token('static'),
    global: $ => token('global'),
  },
});
