/**
 * @file Parser for Cleo programming language
 * @author Daril Rodriguez <me@daril.dev>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: 'cleo',

  // Configuración de palabras clave para evitar que choquen con identificadores
  word: $ => $.ident,

  rules: {
    source_file: $ => repeat($._decl),

    _decl: $ => choice(
      $.fn_decl,
      $.type_decl
    ),

    type_decl: $ => seq(
      'type',
      $.ident,
      optional($.generic_params),
      '=',
      $._type_body
    ),

    _type_body: $ => choice(
      $.struct_decl,
      // $.trait_decl,
      $.enum_decl,
      $.type
    ),

    enum_decl: $ => seq(
      'enum',
      '{',
      optional(seq($.enum_variant, repeat(seq(',', $.enum_variant)), optional(','))),
      '}'
    ),

    enum_variant: $ => $.ident,

    struct_decl: $ => seq(
      'struct',
      '{',
      optional(seq($.struct_field, repeat(seq(',', $.struct_field)), optional(','))),
      '}'
    ),

    struct_field: $ => seq($.ident, ':', $.type),

    _stmt: $ => seq($._expr, optional(';')),

    block: $ => seq(
      '{',
      optional(seq($._stmt, repeat(seq(';', $._stmt)), optional(';'))),
      '}'
    ),

    fn_decl: $ => seq(
      optional('inline'),
      'fn',
      $.ident,
      optional($.generic_params),
      $.fn_params,
      optional($.type),
      $.block
    ),

    fn_params: $ => seq(
      '(',
      optional(seq($.fn_param, repeat(seq(',', $.fn_param)), optional(','))),
      ')'
    ),

    fn_param: $ => seq($.ident, ':', $.type),

    generic_params: $ => seq(
      '[',
      optional(seq($.generic_param, repeat(seq(',', $.generic_param)), optional(','))),
      ']'
    ),

    generic_param: $ => seq($.ident, optional(seq(':', $.type))),

    type: $ => seq(
      repeat($._array_part),
      repeat($._ptr_part),
      $.path
    ),

    _array_part: $ => seq('[', $.usize, ']'),
    _ptr_part: $ => choice('*', seq('*', 'const')),

    path: $ => seq($.segment, repeat(seq('.', $.segment))),

    segment: $ => seq($.ident, optional($.generic_args)),

    generic_args: $ => seq(
      '[',
      optional(seq($.type, repeat(seq(',', $.type)), optional(','))),
      ']'
    ),

    _expr: $ => choice(
      $.if_stmt,
      $.addition
    ),

    // Precedencias para operadores
    addition: $ => prec.left(1, seq(
      $.multiplication,
      repeat(seq(choice('+', '-'), $.multiplication))
    )),

    multiplication: $ => prec.left(2, seq(
      $.division,
      repeat(seq('*', $.division))
    )),

    division: $ => prec.left(2, seq(
      $.unary,
      repeat(seq('/', $.unary))
    )),

    unary: $ => choice(
      seq(choice('*', '&', '!', '-'), $.factor),
      $.factor
    ),

    factor: $ => seq(
      $._primary,
      repeat($._accessor)
    ),

    _primary: $ => choice(
      $.integer,
      $.float,
      $.string,
      $.bool,
      seq('(', $._expr, ')'),
      $.path
    ),

    _accessor: $ => choice(
      $.call,
      $.field_access
    ),

    call: $ => seq(
      '(',
      optional(seq($._expr, repeat(seq(',', $._expr)), optional(','))),
      ')'
    ),

    field_access: $ => seq('.', $.segment),

    if_stmt: $ => seq(
      'if',
      $._expr,
      $.block,
      optional(seq('else', choice($.if_stmt, $.block)))
    ),

    // Definición de tokens básicos
    ident: $ => /[a-zA-Z_]\w*/,
    integer: $ => /\d+/,
    usize: $ => /\d+/,
    float: $ => /\d+\.\d+/,
    string: $ => /"[^"]*"/,
    bool: $ => choice('true', 'false'),
  }
});