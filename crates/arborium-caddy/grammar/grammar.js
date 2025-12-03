const STANDARD_MATCHER_NAMES = [
  'expression',
  'file',
  'header',
  'header_regexp',
  'host',
  'method',
  'not',
  'path',
  'path_regexp',
  'protocol',
  'query',
  'remote_ip',
  'vars',
  'vars_regexp'
]

module.exports = grammar({
  name: 'caddy',

  extras: $ => [/[\t\p{Zs}]/],
  
  externals: $ => [$._eof],

  // word: $ => $._valid_uri_path_string,

  rules: {
    document: $ => repeat(choice(
      $._empty_line,
      $.comment_line,
      $.global_options_block,
      $.snippet,
      $.site_block,
      $.directive_import // Special directive - can appear anywhere in the file!
    )),

    // _horizontal_whitespace: $ => /[\t\p{Zs}]/,
    _horizontal_whitespaces: $ => prec(2, /[\t\p{Zs}]+/),

    _vertical_whitespace: $ => /[\n\r\n\f\x85\x{2028}\x{2029}]/,
    _vertical_whitespaces: $ => /[\n\r\f\x85\x{2028}\x{2029}]+/,

    _whitespace: $ => /[\t\p{Zs}\n\r\n\f\x85\x{2028}\x{2029}]/,
    _whitespaces: $ => /[\t\p{Zs}\n\r\n\f\x85\x{2028}\x{2029}]+/,

    _empty_line: $ => /[\t\p{Zs}]*[\n\r\f\x86\x{2028}\x{2029}]/,

    comment: $ => token(seq('#', /.*/)),
    comment_line: $ => token(seq('#', /.*/, /[\n\r\n\f\x85\x{2028}\x{2029}]/)),

    _newline: _ => choice(/\r/, /\n/, /\r\n/, /\u0085/, /\u000C/, /\u2028/, /\u2029/),

    site_block: $ => seq(
      // TODO: repeatable path
      $.site_address,
      optional(repeat(seq(',', $._horizontal_whitespaces, $.site_address))),
      // TODO: single site block doesn't require '{}' delimiting
      '{',
      repeat(choice(
        $._empty_line,
        $.comment_line,
        $.matcher_definition,
        $.directive_block
      )),
      '}'
    ),

    // TODO: Make sure all of this is handled, like (currently missing) only http://
    // https://caddyserver.com/docs/caddyfile/concepts#addresses
    site_address: $ => seq(
      optional(
        $.protocol,
      ),
      choice(
        seq(
          $._ip_or_domain_address,
          token.immediate(optional(
            /:\d{1,5}/
          ))
        ),
        $._port_number
      )
    ),

    protocol: $ => choice(
      'http://',
      'https://'
    ),

    _ip_or_domain_address: $ => choice(
        'localhost',
        $._ipv4_address,
        $._ipv6_address,
        $.domain_address
    ),

    _ipv4_address: $ => /((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])/,

    _ipv6_address: $ => /(fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|2[0-4][0-9]|1{0,1}[0-9]{0,1}[0-9])\.{3,3})(25[0-5]|2[0-4][0-9]|1{0,1}[0-9]{0,1}[0-9])|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|2[0-4][0-9]|1{0,1}[0-9]{0,1}[0-9])\.{3,3})(25[0-5]|2[0-4][0-9]|1{0,1}[0-9]{0,1}[0-9])|:((:[0-9a-fA-F]{1,4}){1,7}|:))/,

    domain_address: $ => /((([a-zA-Z0-9][a-zA-Z0-9-]{0,61}[a-zA-Z0-9])\.)+)[a-zA-Z]{2,6}/,

    _port_number: $ => seq(
      ':',
      /\d{1,5}/
    ),

    matcher_definition: $ => seq(
      $.matcher_name,
      // TODO: blocks are optional if only one matcher is used
      seq('{', optional($._horizontal_whitespaces)),
      repeat(choice(
        $.comment_line,
        $._empty_line,
        $.standard_matcher
      )),
      '}'
    ),

    matcher_name: $ => token(seq(
      '@',
      /[\w-]*/
    )),
    
    standard_matcher: $ => seq(
      // choice(...STANDARD_MATCHER_NAMES),
      // // TODO: matcher args needs serious work
      // seq(
      //   /[\w\/*]/
      // )
      choice(
        $.matcher_path,
        $.matcher_path_regexp,
      ),
      $._vertical_whitespace
    ),

    matcher_path: $ => seq(
      field('matcher_type', 'path'),
      $._horizontal_whitespaces,
      $.unix_path
    ),
    
    matcher_path_regexp: $ => seq(
      field('matcher_type', 'path_regexp'),
      $._horizontal_whitespaces,
      optional(seq(field('path_regexp_matcher_name', /\w+/), $._horizontal_whitespaces)),
      field('path_regexp_matcher_value', /\S+/)
    ),
    
    directive_block: $ => choice(    
      $.directive_abort,
      // $.directive_acme_server,
      $.directive_basicauth,
      $.directive_bind,
      $.directive_encode,
      $.directive_error,
      $.directive_file_server,
      // $.directive_forward_auth,
      $.directive_handle,
      // $.directive_handle_errors,
      $.directive_handle_path,
      $.directive_header,
      $.directive_import,
      $.directive_log,
      // $.directive_map,
      // $.directive_method,
      $.directive_metrics,
      $.directive_php_fastcgi,
      // $.directive_push,
      $.directive_redir,
      $.directive_request_body,
      // $.directive_request_header,
      $.directive_respond,
      $.directive_reverse_proxy,
      $.directive_rewrite,
      $.directive_root,
      $.directive_route,
      $.directive_skip_log,
      // $.directive_templates,
      $.directive_tls,
      $.directive_tracing,
      // $.directive_try_files,
      // $.directive_uri,
      // $.directive_var
    ),

    directive_abort: $ => seq(
      field('directive_type', 'abort'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      $._vertical_whitespaces
    ),

    directive_basicauth: $ => seq(
      field('directive_type', 'basicauth'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      optional(seq(
        $.hash_algorithm,
        $._horizontal_whitespaces,
        optional(seq(
          field('basicauth_realm_name', /[\w-]+/),
          $._horizontal_whitespaces,
        )),
      )),
      // At least one credential has to be set, so this directive's {} block is not optional
      '{',
      $._vertical_whitespace,
      repeat1(choice(
        $._empty_line,
        $.comment_line,
        $.basicauth_credential
      )),
      '}'
    ),

    hash_algorithm: $ => choice(
      'bcrypt'
      // TODO: list of other algorithms?
    ),

    basicauth_credential: $ => seq(
      field('basicauth_user_name', /\S+/),
      $._horizontal_whitespaces,
      field('basicauth_user_pass', /[a-zA-Z0-9\+\/=\$\.]+/),
      optional(seq(
        $._horizontal_whitespaces,
        field('basicauth_user_pass_saltb64', /[a-zA-Z0-9\+\/=]+/)
      ))
    ),

    directive_bind: $ => seq(
      field('directive_type', 'bind'),
      $._horizontal_whitespaces,
      repeat1($.network_address_wo_port)
    ),

    network_address_wo_port: $ => seq(
      optional(seq(
        $._network,
        '/'
      )),
      choice(
        $._ipv4_address,
        $._ipv6_address,
        // TODO unix socket properly
        $.unix_path
      )
    ),

    network_address_optional_port: $ => seq(
      optional(seq(
        $._network,
        '/'
      )),
      choice(
        seq($._ipv4_address, $._port_number),
        seq($._ipv6_address, $._port_number),
        $.unix_path
      )
    ),

    _network: $ => choice(
      seq(
        choice('tcp', 'udp', 'ip'),
        optional(choice('4', '6')),
      ),
      seq(
        'unix',
        optional(choice('gram', 'packet'))
      )
    ),

    unix_path: $ => choice(
      seq(
        '"',
        /[\w\s\/\.\*-]+/,
        '"'
      ),
      /[\w\/\.\*-]+/
    ),

    directive_encode: $ => seq(
      field('directive_type', 'encode'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      repeat1($.encode_format)
    ),

    encode_format: $ => choice('gzip', 'zstd'),

    directive_file_server: $ => seq(
      field('directive_type', 'file_server'),
      optional(seq( $._horizontal_whitespaces, $.matcher_token)),
      optional(seq(
        choice(
          seq(
            repeat1(seq($._horizontal_whitespaces, field('file_server_option_browse', 'browse'))),
            $._vertical_whitespace
          ),
          seq(
          '{',
            // TODO: file_server options
          repeat1(choice(
            $._empty_line,
            $.comment_line,
            field('file_server_option_browse', 'browse')
          )),
          '}'
          )
        )
      )),
      $._vertical_whitespace
    ),
    
    directive_error: $ => seq(
      field('directive_type', 'error'),
      optional(seq($._horizontal_whitespaces, $.matcher_token)),
      choice(
        seq(
          optional(seq($._horizontal_whitespaces, $.http_message)),
          optional(seq($._horizontal_whitespaces, $.http_status_code)),
          $._vertical_whitespace
        ),
        seq(
          optional($.http_error_code),
          '{',
          repeat1(choice(
            $._empty_line,
            $.comment_line,
            seq(
              field('directive_option_name', 'message'),
              $._horizontal_whitespaces,
              field('respond_or_error_message', $.http_message),
            ),
          )),
          '}'
        )
      )
    ),

    http_message: $ => choice(/"[\w -]+"/, /`[\w\{\}\[\]\/:,\." -]+`/),

    http_error_code: $ => /[4|5]\d{2}/,

    directive_handle: $ => seq(
      field('directive_type', 'handle'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      // At least one child directive has to be set, so this directive's {} block is not optional
      '{',
      repeat1(choice(
        $._empty_line,
        $.comment_line,
        $.directive_block,
      )),
      '}'
    ),

    directive_handle_path: $ => seq(
      field('directive_type', 'handle_path'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      // At least one child directive has to be set, so this directive's {} block is not optional
      '{',
      repeat1(choice(
        $._empty_line,
        $.comment_line,
        $.directive_block,
      )),
      '}'
    ),
    
    directive_header: $ => seq(
      field('directive_type', 'header'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      choice(
        $.field_manipulator,
        seq(
          '{',
          repeat1(choice(
            $._empty_line,
            $.comment_line,
            $.field_manipulator,
            'defer', $._vertical_whitespace
          )),
          '}',
        )
      )
    ),

    field_manipulator: $ => choice(
      $.field_replace,
      $.field_add_or_set,
      $.field_delete,
      $.field_set_default
    ),

    field_replace: $ => seq(
      $.header_name,
      $._horizontal_whitespaces,
      $.header_value,
      $._horizontal_whitespaces,
      $.header_value
    ),

    field_add_or_set: $ => seq(
      optional('+'),
      $.header_name,
      $._horizontal_whitespaces,
      $.header_value
    ),

    field_delete: $ => seq(
      '-',
      $.header_name
    ),

    field_set_default: $ => seq(
      '?',
      $.header_name,
      $._horizontal_whitespaces,
      $.header_value
    ),

    header_name: $ => /[\w-]+/,

    header_value: $ => choice(
      /[\w:;\.,\\\/'\?\!\(\){}\[\]@<>=+\*#\$\&`\|\~\^%\-]+/,
      seq(
        '"',
        /[\w:;\.,\\\/'\?\!\(\){}\[\]@<>=+\*#\$\&`\|\~\^% \-]+/,
        '"'
      ),
      seq(
        "'",
        /[\w:;\.,\\\/"\?\!\(\){}\[\]@<>=+\*#\$\&`\|\~\^% \-]+/,
        "'" 
      )
    ),

    directive_import: $ => seq(
      field('directive_type', 'import'),
      $._horizontal_whitespaces,
      choice(
        $.snippet_name
        // TODO: file import?
      ),
      // TODO: optional args to pass
      $._vertical_whitespace
    ),

    directive_log: $ => seq(
      field('directive_type', 'log'),
      $._horizontal_whitespaces,
      optional(seq(
        '{',
        repeat(choice(
          $._empty_line,
          $.comment_line,
          $.log_option_output,
          $.log_option_format,
          $.log_option_level
        )),
        '}'
      ))
    ),

    log_option_output: $ => seq(
      field('directive_option_name', 'output'),
      $._horizontal_whitespaces,
      choice(
        seq(choice('stderr', 'stdout', 'discard'), $._vertical_whitespace),
        $._log_output_file,
      )
    ),

    _log_output_file: $ => seq(
      field('directive_option_fixed_value','file'),
      $._horizontal_whitespaces,
      $.unix_path,
      choice(
        $._vertical_whitespace,
        seq(
          $._horizontal_whitespaces,
          '{',
          repeat(choice(
            $._empty_line,
            $.comment_line,
            'roll_disabled',
            seq('roll_size', $._horizontal_whitespaces, $.size_number),
            'roll_uncompressed',
            'roll_local_time',
            seq('roll_keep', $._horizontal_whitespaces, /\d+/),
            seq('roll_keep_for_days', $._horizontal_whitespaces, /\d+[a-zA-Z]*/),
            )),
          '}'
        )
      )
    ),

    size_number: $ => /\d+(\.\d+)?[a-zA-Z]*/,

    log_option_format: $ => seq(
      field('directive_option_name', 'format'),
      $._horizontal_whitespaces,
      choice(
        seq(field('directive_option_fixed_value', choice('console', 'json')), $._vertical_whitespace),
        //TODO: filter format module
      ),
      // TODO: format options
    ),

    log_option_level: $ => seq(
      field('directive_option_name', 'level'),
      $._horizontal_whitespaces,
      field('directive_option_fixed_value', choice('INFO', 'info', 'ERROR', 'error')),
      $._vertical_whitespace
    ),
    
    directive_redir: $ => seq(
      field('directive_type', 'redir'),
      optional(seq($._horizontal_whitespaces, $.matcher_token)),
      $._horizontal_whitespaces,
      $.redir_or_rewrite_target,
      optional(seq($._horizontal_whitespaces, $.redir_code)),
      $._vertical_whitespace
    ),

    directive_php_fastcgi: $ => seq(
      field('directive_type', 'php_fastcgi'),
      optional(seq($._horizontal_whitespaces, $.matcher_token)),
      $._horizontal_whitespaces,
      $.network_address_optional_port,
      optional(seq(
        '{',
        repeat1(choice(
          $._empty_line,
          $.comment_line,
          $.fastcgi_option_root,
          $.fastcgi_option_split,
          $.fastcgi_option_env,
          $.fastcgi_option_index,
          $.fastcgi_option_try_files,
          $.fastcgi_option_resolve_root_symlink,
          $.fastcgi_option_dial_timeout,
          $.fastcgi_option_read_timeout,
          $.fastcgi_option_write_timeout,
          $.fastcgi_option_capture_stderr,
          $.reverse_proxy_option_header_up,
          $.reverse_proxy_option_header_down
        )),
        '}',
      ))
    ),

    fastcgi_option_root: $ => seq(field('directive_option_name', 'root'),
      $._horizontal_whitespaces,
      $.unix_path,
      $._vertical_whitespace
    ),

    fastcgi_option_split: $ => seq(field('directive_option_name', 'split'),
      $._horizontal_whitespaces,
      /\S+/,
      $._vertical_whitespace
    ),

    fastcgi_option_env: $ => seq(field('directive_option_name', 'env'),
      $._horizontal_whitespaces,
      /\S+/,
      $._horizontal_whitespaces,
      /\S+/,
      $._vertical_whitespace
    ),

    fastcgi_option_index: $ => seq(field('directive_option_name', 'index'),
      $._horizontal_whitespaces,
      choice('off', $.unix_path),
      $._vertical_whitespace
    ),

    fastcgi_option_try_files: $ => seq(field('directive_option_name', 'try_files'),
      $._horizontal_whitespaces,
      repeat1($.unix_path), // TODO: add placeholder
      $._vertical_whitespace
    ),

    fastcgi_option_resolve_root_symlink: $ => seq(field('directive_option_name', 'resolve_root_symlink'), $._vertical_whitespace),
    
    fastcgi_option_capture_stderr: $ => seq(field('directive_option_name', 'capture_stderr'), $._vertical_whitespace),
    
    fastcgi_option_dial_timeout: $ => seq(field('directive_option_name', 'dial_timeout'),
      $._horizontal_whitespaces,
      $.duration_value,
      $._vertical_whitespace
    ),
    
    fastcgi_option_read_timeout: $ => seq(field('directive_option_name', 'read_timeout'),
      $._horizontal_whitespaces,
      $.duration_value,
      $._vertical_whitespace
    ),
    
    fastcgi_option_write_timeout: $ => seq(field('directive_option_name', 'write_timeout'),
      $._horizontal_whitespaces,
      $.duration_value,
      $._vertical_whitespace
    ),

    duration_value: $ => /[0-9\.]+(((n|u|m)?s)|m|h|d)/,

    directive_metrics: $ => seq(
      field('directive_type', 'metrics'),
      optional(seq($._horizontal_whitespaces, $.matcher_token)),
      optional(seq(
        $._horizontal_whitespaces,
        '{',
        repeat1(choice(
          $._empty_line,
          $.comment_line,
          field('directive_option_name', 'disable_openmetrics')
        )),
        '}'
      )),
      $._vertical_whitespace
    ),

    directive_request_body: $ => seq(
      field('directive_type', 'request_body'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      '{',
      optional(repeat(choice(
        $._empty_line,
        $.comment_line,
      ))),
      // TODO: validate size literal?
      $.request_body_option_max_size,
      optional(repeat(choice(
        $._empty_line,
        $.comment_line,
      ))),
      optional($._vertical_whitespace),
      '}',
    ),

    request_body_option_max_size: $ => seq(field('directive_option_name', 'max_size'), $.size_number, $._vertical_whitespace),
    
    redir_or_rewrite_target: $ => choice(
      field('address_target', seq(
        optional($.protocol),
        $.domain_address,
        optional($.uri_path_with_placeholders)
      )),
      field('uri_path_target', $.uri_path_with_placeholders)
    ),

    directive_respond: $ => seq(
      field('directive_type', 'respond'),
      optional(seq($._horizontal_whitespaces, $.matcher_token)),
      choice(
        seq(
          optional(seq($._horizontal_whitespaces, $.http_message)),
          optional(seq($._horizontal_whitespaces, $.http_status_code)),
          $._vertical_whitespace
        ),
        seq(
          optional(seq($._horizontal_whitespaces, $.http_status_code)),
          '{',
          repeat1(choice(
            $._empty_line,
            $.comment_line,
            seq(
              field('directive_option_name', 'body'),
              $._horizontal_whitespaces,
              field('respond_or_error_message', $.http_message),
            ),
            field('directive_option_name', 'close')
          )),
          '}'
        )
      )
    ),

    http_status_code: $ => /\d{3}/,

    directive_reverse_proxy: $ => seq(
      field('directive_type', 'reverse_proxy'),
      optional(seq($._horizontal_whitespaces, $.matcher_token)),
      repeat1(seq($._horizontal_whitespaces, choice($.site_address, $.network_address_optional_port))),
      // TODO: multiple address, and options block
      optional(seq(
        $._horizontal_whitespaces,
        '{',
        repeat1(choice(
          $._empty_line,
          $.comment_line,
          $.reverse_proxy_option_trusted_proxies,
          $.reverse_proxy_option_header_up,
          $.reverse_proxy_option_header_down
        )),
        '}'
      ))
    ),

    reverse_proxy_option_trusted_proxies: $ => seq(
      field('directive_option_name', 'trusted_proxies'),
      repeat1(choice(
        seq($._horizontal_whitespaces, $._ipv4_address),
        seq($._horizontal_whitespaces, $._ipv6_address),
        seq($._horizontal_whitespaces, $._ipv4_address, '-', $._ipv4_address),
        seq($._horizontal_whitespaces, $._ipv6_address, '-', $._ipv6_address)
      )),
      $._vertical_whitespace
    ),

    reverse_proxy_option_header_up: $ => seq(field(
      'directive_option_name', 'header_up'),
      $._horizontal_whitespaces,
      $.field_manipulator, $._vertical_whitespace
    ),
    
    reverse_proxy_option_header_down: $ => seq(field('directive_option_name', 'header_down'),
      $._horizontal_whitespaces,
      $.field_manipulator,
      $._vertical_whitespace
    ),
    
    // TODO: When can/must start with leading '/'?
    uri_path_with_placeholders: $ => token(repeat1(
      choice(
        '/',
        /([\w\.\~\!\$\&'\(\)\*\+,;=:@-]|(%[0-9a-fA-F]{2}))+/,
        /\{[a-z\.]+\}/,
      )
    )),

    _valid_uri_path_string: $ => /\/([\w\.\~\!\$\&'\(\)\*\+,;=:@\/-]|(%[0-9a-fA-F]{2}))*/,
      
    redir_code: $ => choice(/3\d\d/, '401', 'temporary', 'permanent', 'html'),

    directive_rewrite: $ => seq(
      field('directive_type', 'rewrite'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      $.redir_or_rewrite_target
    ),

    directive_root: $ => seq(
      field('directive_type', 'root'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      $.unix_path
    ),
    
    directive_tls: $ => seq(
      field('directive_type', 'tls'),
      $._horizontal_whitespaces,
      optional(choice('internal', $.email_address)),
      optional(seq(
        field('cert_file', $.unix_path),
        $._horizontal_whitespaces,
        // token.immediate(prec(2, /[\t\p{Zs}]+/)),
        field('key_file', $.unix_path)
      )),
      // $._vertical_whitespace
      // optional(seq(
      //   '{',
      //   // TODO: tls options
      //   '}'
      // ))
    ),

    directive_tracing: $ => seq(
      field('directive_type', 'tracing'),
      optional(seq(
        $._horizontal_whitespaces,
        '{',
        repeat1(choice(
          $._empty_line,
          $.comment_line,
          seq(
            field('directive_option_name', 'span'),
            $._horizontal_whitespaces,
            field('directive_option_user_string_value', $.uri_path_with_placeholders)
          )
        )),
        '}'
      ))
    ),

    directive_route: $ => seq(
      field('directive_type', 'route'),
      $._horizontal_whitespaces,
      optional(seq($.matcher_token, $._horizontal_whitespaces)),
      // At least one child directive has to be set, so this directive's {} block is not optional
      '{',
      repeat1(choice(
        $._empty_line,
        $.comment_line,
        $.directive_block,
      )),
      '}'
    ),

    directive_skip_log: $ => seq(
      field('directive_type', 'skip_log'),
      optional(seq($._horizontal_whitespaces, $.matcher_token)),
      $._vertical_whitespace
    ),

    email_address: $ => /(?:[a-z0-9!#$%&'*+\/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+\/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])/,
    // _email_address: $ => /[\w.-]+@[\w.-]+/,

    matcher_token: $ => choice(
      '*',
      field('path_matcher', $._valid_uri_path_string),
      $.matcher_name
    ),
    
    subdirective_block: $ => choice(
      // $.directive_name,
      $.subdirective_name,
    ),

    // TODO: external scanner to validate if subdirective is valid for directive?
    subdirective_name: $ => /\w+/,
    // directive_arg: $ => /.*/

    global_options_block: $ => seq(
      '{',
      repeat(choice(
        $._empty_line,
        $.comment_line,
        // TODO: actual global options
        $.global_option_email,
        $.global_option_auto_https
      )),
      '}'
    ),

    global_option_email: $ => seq(
      field('global_option_type', 'email'),
      $._horizontal_whitespaces,
      $.email_address,
      $._vertical_whitespace
    ),

    global_option_auto_https: $ => seq(
      field('global_option_type', 'auto_https'),
      $._horizontal_whitespaces,
      choice('off', 'disable_redirects', 'ignore_loaded_certs', 'disable_certs')
    ),

    snippet: $ => seq(
      $.snippet_name,
      '{',
      // TODO: at least one non-empty_OR_comment-line should be inside
      // repeat(choice(4-options)), repeat1(choice(matcher-or-directive)), repeat(choice(4-options))
      // not too pretty...
      repeat(choice(
        $._empty_line,
        $.comment_line,
        $.matcher_definition,
        $.directive_block
      )),
      optional($._vertical_whitespaces),
      '}',
      $._vertical_whitespace
    ),

    // TODO: Are numbers allowed?
    snippet_name: $ => token(seq('(', /[\w-]*/, ')')),

    
  }
});
