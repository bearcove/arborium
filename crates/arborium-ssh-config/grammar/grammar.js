/**
 * @file Tree-sitter grammar definition
 * @author ObserverOfTime
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

/**
 * @param word {string}
 * @param name {'keyword'|'criteria'}
 */
const keyword = (word, name = 'keyword') =>
  field(name, alias(new RegExp(word, 'i'), word));

/**
 * @param arg {RuleOrLiteral}
 */
const argument = (arg) => field('argument', arg);

/**
 * @param sep {','|SymbolRule<'_space'>}
 * @param rule {Rule}
 */
const list = (sep, rule) =>
  prec.right(seq(rule, repeat(seq(sep, rule))));

/**
 * @param content {RuleOrLiteral}
 * @param extra {...Rule}
 */
const pattern = (content, ...extra) =>
  repeat1(choice('*', '?', content, ...extra));

/**
 * @param content {RuleOrLiteral}
 * @param token {Rule}
 * @param extra {...Rule}
 */
const token_ = (content, token, ...extra) =>
  repeat1(choice(content, token, ...extra));

/**
 * @param prefix {'+-'|'+-^'}
 * @param option {SymbolRule<'cipher'|'kex'|'key_sig'|'mac'|'sig'>}
 */
const algorithms = (prefix, option) =>
  argument(seq(
    optional(choice(...prefix)),
    list(',', option)
  ));

/**
 * @param content {RuleOrLiteral}
 * @param rule {SymbolRule<'number'>}
 */
const override = (content, rule) =>
  seq(
    field('file', pattern(content)),
    ':',
    field('function', pattern(content)),
    ':',
    field('line', choice('*', rule))
  );

module.exports = grammar({
  name: 'ssh_config',

  extras: _ => [],

  conflicts: $ => [
    [$._file_string, $._string],
    [$._file_string, $._plain_string]
  ],

  inline: $ => [
    $._add_keys_to_agent_arg,
    $._control_master_arg,
    $._control_persist_arg,
    $._forward_agent_arg,
    $._identity_agent_arg,
    $._ipqos_arg,
    $._proxy_command_arg,
    $._request_tty_arg,
    $._security_key_provider_arg,
    $._strict_host_key_checking_arg,
    $._tunnel_arg,
    $._tunnel_device_arg,
  ],

  rules: {
    config: $ => repeat(choice(
      seq(
        optional($._space),
        optional($.comment),
        $._eol
      ),
      seq(
        optional($._space),
        $.host_declaration
      ),
      seq(
        optional($._space),
        $.match_declaration
      ),
      seq(
        optional($._space),
        $.parameter,
        optional($._space),
        $._eol
      )
    )),

    host_declaration: $ => prec.right(seq(
      keyword('Host'),
      $._sep,
      list($._space, argument(
        seq(optional('!'), $._pattern)
      )),
      optional($._space),
      $._eol,
      $._declarations
    )),

    match_declaration: $ => prec.right(seq(
      keyword('Match'),
      $._sep,
      choice(
        $._all,
        list($._space, $.condition)
      ),
      optional($._space),
      $._eol,
      optional($._declarations)
    )),

    condition: $ => seq(
      optional('!'),
      choice(
        $._match_canonical,
        $._match_final,
        $._match_exec,
        $._match_localnetwork,
        $._match_host,
        $._match_originalhost,
        $._match_tagged,
        $._match_user,
        $._match_localuser
      )
    ),

    _all: _ => alias(/[aA][lL][lL]/, 'all'),

    _match_canonical: $ => prec.right(seq(
      keyword('canonical', 'criteria'),
      optional(seq($._sep, $._all))
    )),

    _match_final: $ => prec.right(seq(
      keyword('final', 'criteria'),
      optional(seq($._sep, $._all))
    )),

    _match_exec: $ => seq(
      keyword('exec', 'criteria'),
      $._sep,
      argument(choice(
        alias(token_(/\S/, $._file_token), $.string),
        seq('"', alias(token_($._char, $._file_token), $.string), '"')
      ))
    ),

    _match_localnetwork: $ => seq(
      keyword('localnetwork', 'criteria'),
      $._sep,
      argument(choice(
        list(',', alias(/\S+/, $.string)),
        seq('"', list(',', alias(repeat1($._char), $.string)), '"')
      ))
    ),

    _match_host: $ => seq(
      keyword('host', 'criteria'),
      $._sep,
      argument($._match_value)
    ),

    _match_originalhost: $ => seq(
      keyword('originalhost', 'criteria'),
      $._sep,
      argument($._match_value)
    ),

    _match_tagged: $ => seq(
      keyword('tagged', 'criteria'),
      $._sep,
      argument($._match_value)
    ),

    _match_user: $ => seq(
      keyword('user', 'criteria'),
      $._sep,
      argument($._match_value)
    ),

    _match_localuser: $ => seq(
      keyword('localuser', 'criteria'),
      $._sep,
      argument($._match_value)
    ),

    _match_value: $ => choice(
      list(',', alias(pattern(/\S/), $.pattern)),
      seq('"', list(',', alias(pattern($._char), $.pattern)), '"')
    ),

    _declarations: $ => prec.right(
      repeat1(seq(
        optional($._space),
        choice($.comment, $.parameter),
        optional($._space),
        $._eol
      ))
    ),

    parameter: $ => choice(
      $._add_keys_to_agent,
      $._address_family,
      $._batch_mode,
      $._bind_address,
      $._bind_interface,
      $._canonical_domains,
      $._canonicalize_fallback_local,
      $._canonicalize_hostname,
      $._canonicalize_max_dots,
      $._canonicalize_permitted_cnames,
      $._ca_signature_algorithms,
      $._certificate_file,
      $._check_host_ip,
      $._ciphers,
      $._clear_all_forwardings,
      $._compression,
      $._connection_attempts,
      $._connect_timeout,
      $._control_master,
      $._control_path,
      $._control_persist,
      $._dynamic_forward,
      $._enable_escape_command_line,
      $._enable_ssh_keysign,
      $._escape_char,
      $._exit_on_forward_failure,
      $._fingerprint_hash,
      $._fork_after_authentication,
      $._forward_agent,
      $._forward_x11,
      $._forward_x11_timeout,
      $._forward_x11_trusted,
      $._global_known_hosts_file,
      $._gssapi_authentication,
      $._gssapi_delegate_credentials,
      $._hash_known_hosts,
      $._hostbased_accepted_algorithms,
      $._hostbased_authentication,
      $._host_key_algorithms,
      $._host_key_alias,
      $._hostname,
      $._identities_only,
      $._identity_agent,
      $._identity_file,
      $._ignore_unknown,
      $._include,
      $._ipqos,
      $._kbd_interactive_authentication,
      $._kex_algorithms,
      $._known_hosts_command,
      $._local_command,
      $._local_forward,
      $._log_level,
      $._log_verbose,
      $._macs,
      $._no_host_authentication_for_localhost,
      $._number_of_password_prompts,
      $._password_authentication,
      $._permit_local_command,
      $._permit_remote_open,
      $._pkcs11_provider,
      $._port,
      $._preferred_authentications,
      $._proxy_command,
      $._proxy_jump,
      $._proxy_use_fdpass,
      $._pubkey_accepted_algorithms,
      $._pubkey_authentication,
      $._rekey_limit,
      $._remote_command,
      $._remote_forward,
      $._request_tty,
      $._required_rsa_size,
      $._revoked_host_keys,
      $._security_key_provider,
      $._send_env,
      $._server_alive_count_max,
      $._server_alive_interval,
      $._session_type,
      $._set_env,
      $._stdin_null,
      $._stream_local_bind_mask,
      $._stream_local_bind_unlink,
      $._strict_host_key_checking,
      $._syslog_facility,
      $._tcp_keep_alive,
      $._tag,
      $._tunnel,
      $._tunnel_device,
      $._update_host_keys,
      $._use_keychain,
      $._user,
      $._user_known_hosts_file,
      $._verify_host_key_dns,
      $._visual_host_key,
      $._xauth_location,
    ),

    _add_keys_to_agent: $ => seq(
      keyword('AddKeysToAgent'),
      $._sep,
      argument($._add_keys_to_agent_arg)
    ),

    _add_keys_to_agent_arg: $ => choice(
      $._boolean,
      'ask',
      prec.right(seq(
        'confirm',
        optional(seq($._space, $.time))
      )),
      $.time
    ),

    _address_family: $ => seq(
      keyword('AddressFamily'),
      $._sep,
      argument(choice('any', 'inet', 'inet6'))
    ),

    _batch_mode: $ => seq(
      keyword('BatchMode'),
      $._sep,
      argument($._boolean)
    ),

    _bind_address: $ => seq(
      keyword('BindAddress'),
      $._sep,
      argument($._pattern)
    ),

    _bind_interface: $ => seq(
      keyword('BindInterface'),
      $._sep,
      argument($._pattern)
    ),

    _canonical_domains: $ => seq(
      keyword('CanonicalDomains'),
      $._sep,
      list($._space, argument($._pattern))
    ),

    _canonicalize_fallback_local: $ => seq(
      keyword('CanonicalizeFallbackLocal'),
      $._sep,
      argument($._boolean)
    ),

    _canonicalize_hostname: $ => seq(
      keyword('CanonicalizeHostname'),
      $._sep,
      argument(choice('always', $._boolean))
    ),

    _canonicalize_max_dots: $ => seq(
      keyword('CanonicalizeMaxDots'),
      $._sep,
      argument($.number)
    ),

    _canonicalize_permitted_cnames: $ => seq(
      keyword('CanonicalizePermittedCNAMEs'),
      $._sep,
      choice(
        'none',
        list($._space, argument($._cnames_map))
      )
    ),

    _cnames_map: $ => seq(
      field('source_domain_list', list(',', $._pattern)),
      ':',
      field('target_domain_list', list(',', $._pattern)),
    ),

    _ca_signature_algorithms: $ => seq(
      keyword('CASignatureAlgorithms'),
      $._sep,
      algorithms('+-', $.sig)
    ),

    _certificate_file: $ => seq(
      keyword('CertificateFile'),
      $._sep,
      argument($._file_pattern_vars)
    ),

    _check_host_ip: $ => seq(
      keyword('CheckHostIP'),
      $._sep,
      argument($._boolean)
    ),

    _ciphers: $ => seq(
      keyword('Ciphers'),
      $._sep,
      algorithms('+-^', $.cipher)
    ),

    _clear_all_forwardings: $ => seq(
      keyword('ClearAllForwardings'),
      $._sep,
      argument($._boolean)
    ),

    _compression: $ => seq(
      keyword('Compression'),
      $._sep,
      argument($._boolean)
    ),

    _connection_attempts: $ => seq(
      keyword('ConnectionAttempts'),
      $._sep,
      argument($.number)
    ),

    _connect_timeout: $ => seq(
      keyword('ConnectTimeout'),
      $._sep,
      argument($.number)
    ),

    _control_master: $ => seq(
      keyword('ControlMaster'),
      $._sep,
      argument($._control_master_arg)
    ),

    _control_master_arg: $ => choice(
      $._boolean,
      'ask',
      'auto',
      'autoask'
    ),

    _control_persist: $ => seq(
      keyword('ControlPersist'),
      $._sep,
      argument($._control_persist_arg)
    ),

    _control_persist_arg: $ => choice(
      $._boolean,
      $.time
    ),

    _control_path: $ => seq(
      keyword('ControlPath'),
      $._sep,
      argument($._file_pattern_vars)
    ),

    _dynamic_forward: $ => seq(
      keyword('DynamicForward'),
      $._sep,
      list($._space, argument($._dynamic_forward_value))
    ),

    _forward_value_inner: $ => seq(
        field('bind_address', choice('*', $._string)),
        ':',
        field('port', $.number)
    ),

    _dynamic_forward_value: $ => choice(
      field('port', $.number),
      $._forward_value_inner,
      seq('"', $._forward_value_inner, '"')
    ),

    _enable_escape_command_line: $ => seq(
      keyword('EnableEscapeCommandline'),
      $._sep,
      argument($._boolean)
    ),

    _enable_ssh_keysign: $ => seq(
      keyword('EnableSSHKeysign'),
      $._sep,
      argument($._boolean)
    ),

    _escape_char: $ => seq(
      keyword('EscapeChar'),
      $._sep,
      argument(choice(/\S|\^[A-Za-z]/, 'none'))
    ),

    _exit_on_forward_failure: $ => seq(
      keyword('ExitOnForwardFailure'),
      $._sep,
      argument($._boolean)
    ),

    _fingerprint_hash: $ => seq(
      keyword('FingerprintHash'),
      $._sep,
      argument(choice('md5', 'sha256'))
    ),

    _fork_after_authentication: $ => seq(
      keyword('ForkAfterAuthentication'),
      $._sep,
      argument($._boolean)
    ),

    _forward_agent: $ => seq(
      keyword('ForwardAgent'),
      $._sep,
      argument($._forward_agent_arg)
    ),

    _forward_agent_arg: $ => choice(
      $._boolean,
      $._string,
      $._var_value
    ),

    _forward_x11: $ => seq(
      keyword('ForwardX11'),
      $._sep,
      argument($._boolean)
    ),

    _forward_x11_timeout: $ => seq(
      keyword('ForwardX11Timeout'),
      $._sep,
      argument($.time)
    ),

    _forward_x11_trusted: $ => seq(
      keyword('ForwardX11Trusted'),
      $._sep,
      argument($._boolean)
    ),

    _global_known_hosts_file: $ => seq(
      keyword('GlobalKnownHostsFile'),
      $._sep,
      list($._space, argument($._string))
    ),

    _gssapi_authentication: $ => seq(
      keyword('GSSAPIAuthentication'),
      $._sep,
      argument($._boolean)
    ),

    _gssapi_delegate_credentials: $ => seq(
      keyword('GSSAPIDelegateCredentials'),
      $._sep,
      argument($._boolean)
    ),

    _hash_known_hosts: $ => seq(
      keyword('HashKnownHosts'),
      $._sep,
      argument($._boolean)
    ),

    _hostbased_accepted_algorithms: $ => seq(
      choice(
        keyword('HostbasedAcceptedAlgorithms'),
        keyword('HostbasedKeyTypes')
      ),
      $._sep,
      algorithms('+-^', $.key_sig)
    ),

    _hostbased_authentication: $ => seq(
      keyword('HostbasedAuthentication'),
      $._sep,
      argument($._boolean)
    ),

    _host_key_algorithms: $ => seq(
      keyword('HostKeyAlgorithms'),
      $._sep,
      algorithms('+-^', $.key_sig)
    ),

    _host_key_alias: $ => seq(
      keyword('HostKeyAlias'),
      $._sep,
      argument($._string)
    ),

    _hostname: $ => seq(
      keyword('Hostname'),
      $._sep,
      argument($._hostname_string)
    ),

    _identities_only: $ => seq(
      keyword('IdentitiesOnly'),
      $._sep,
      argument($._boolean)
    ),

    _identity_agent: $ => seq(
      keyword('IdentityAgent'),
      $._sep,
      argument($._identity_agent_arg)
    ),

    _identity_agent_arg: $ => choice(
      'none',
      'SSH_AUTH_SOCK',
      $._file_string,
      $._var_value
    ),

    _identity_file: $ => seq(
      keyword('IdentityFile'),
      $._sep,
      argument($._file_string)
    ),

    _ignore_unknown: $ => seq(
      keyword('IgnoreUnknown'),
      $._sep,
      argument(list($._space, $._pattern))
    ),

    _include: $ => seq(
      keyword('Include'),
      $._sep,
      argument($._pattern)
    ),

    _ipqos: $ => prec.right(seq(
      keyword('IPQoS'),
      $._sep,
      argument($._ipqos_arg),
      optional(seq($._space, argument($._ipqos_arg)))
    )),

    _ipqos_arg: $ => choice(
      $.ipqos,
      $.number,
      'none'
    ),

    _kbd_interactive_authentication: $ => seq(
      choice(
        keyword('KbdInteractiveAuthentication'),
        keyword('ChallengeResponseAuthentication')
      ),
      $._sep,
      argument($._boolean)
    ),

    _kex_algorithms: $ => seq(
      keyword('KexAlgorithms'),
      $._sep,
      algorithms('+-^', $.kex)
    ),

    _known_hosts_command: $ => seq(
      keyword('KnownHostsCommand'),
      $._sep,
      argument($._hosts_string)
    ),

    _local_command: $ => seq(
      keyword('LocalCommand'),
      $._sep,
      argument($._token_string)
    ),

    _local_forward: $ => seq(
      keyword('LocalForward'),
      $._sep,
      argument($._forward_value1),
      $._space,
      argument($._forward_value2)
    ),

    _forward_value1: $ => choice(
      field('socket', $._file_string),
      field('port', $.number),
      $._forward_value_inner,
      seq('"', $._forward_value_inner, '"')
    ),

    _forward_value2: $ => choice(
      field('socket', $._file_string),
      $._forward_value_inner,
      seq('"', $._forward_value_inner, '"')
    ),

    _log_level: $ => seq(
      keyword('LogLevel'),
      $._sep,
      argument($.verbosity)
    ),

    _log_verbose: $ => seq(
      keyword('LogVerbose'),
      $._sep,
      argument(choice(
        list(',', $._log_verbose_value),
        seq('"', list(',', $._log_verbose_quoted), '"')
      ))
    ),

    _log_verbose_value: $ =>
      override(/S/, $.number),

    _log_verbose_quoted: $ =>
      override($._char, $.number),

    _macs: $ => seq(
      keyword('MACs'),
      $._sep,
      algorithms('+-^', $.mac)
    ),

    _no_host_authentication_for_localhost: $ => seq(
      keyword('NoHostAuthenticationForLocalhost'),
      $._sep,
      argument($._boolean)
    ),

    _number_of_password_prompts: $ => seq(
      keyword('NumberOfPasswordPrompts'),
      $._sep,
      argument($.number)
    ),

    _password_authentication: $ => seq(
      keyword('PasswordAuthentication'),
      $._sep,
      argument($._boolean)
    ),

    _permit_local_command: $ => seq(
      keyword('PermitLocalCommand'),
      $._sep,
      argument($._boolean)
    ),

    _permit_remote_open: $ => seq(
      keyword('PermitRemoteOpen'),
      $._sep,
      argument(list($._space, $._permit_remote_open_value))
    ),

    _permit_remote_open_value: $ => choice(
      'any',
      'none',
      seq(
        field('host', choice('*', alias(/\S+/, $.string))),
        ':',
        field('port', choice('*', $.number))
      ),
      seq(
        '"',
        field('host', choice('*', alias(repeat1($._char), $.string))),
        ':',
        field('port', choice('*', $.number)),
        '"'
      ),
    ),

    _pkcs11_provider: $ => seq(
      keyword('PKCS11Provider'),
      $._sep,
      argument($._string)
    ),

    _port: $ => seq(
      keyword('Port'),
      $._sep,
      argument($.number)
    ),

    _preferred_authentications: $ => seq(
      keyword('PreferredAuthentications'),
      $._sep,
      argument(list(',', $.authentication))
    ),

    _proxy_command: $ => seq(
      keyword('ProxyCommand'),
      $._sep,
      argument($._proxy_command_arg)
    ),

    _proxy_command_arg: $ => choice(
      'none',
      $._proxy_string
    ),

    _proxy_jump: $ => seq(
      keyword('ProxyJump'),
      $._sep,
      list(',', $._proxy_jump_value)
    ),

    _proxy_jump_value: $ => choice(
      argument('none'),
      seq(
        optional(seq(
          field('user', $._plain_string),
          '@'
        )),
        field('host', $._plain_string),
        optional(seq(
          ':',
          field('port', $.number)
        ))
      ),
      field('uri', alias(/ssh:\/\/\S+/, $.uri))
    ),

    _proxy_use_fdpass: $ => seq(
      keyword('ProxyUseFdpass'),
      $._sep,
      argument($._boolean)
    ),

    _pubkey_accepted_algorithms: $ => seq(
      choice(
        keyword('PubkeyAcceptedAlgorithms'),
        keyword('PubkeyAcceptedKeyTypes')
      ),
      $._sep,
      algorithms('+-^', $.key_sig)
    ),

    _pubkey_authentication: $ => seq(
      keyword('PubkeyAuthentication'),
      $._sep,
      argument($._pubkey_authentication_arg)
    ),

    _pubkey_authentication_arg: $ => choice(
      $._boolean,
      'unbound',
      'host-bound'
    ),

    _rekey_limit: $ => prec.right(seq(
      keyword('RekeyLimit'),
      $._sep,
      choice(
        argument('none'),
        argument($.bytes),
        seq(
          argument($.bytes),
          $._space,
          argument($.time)
        )
      )
    )),

    _remote_command: $ => seq(
      keyword('RemoteCommand'),
      $._sep,
      argument(alias(
        token_(/[^\r\n]/, $._file_token, $.variable),
        $.string
      ))
    ),

    _remote_forward: $ => prec.right(seq(
      keyword('RemoteForward'),
      $._sep,
      argument($._forward_value1),
      optional(seq(
        $._space,
        argument($._forward_value2)
      ))
    )),

    _request_tty: $ => seq(
      keyword('RequestTTY'),
      $._sep,
      argument($._request_tty_arg)
    ),

    _request_tty_arg: $ => choice(
      $._boolean,
      'force',
      'auto'
    ),

    _required_rsa_size: $ => seq(
      keyword('RequiredRSASize'),
      $._sep,
      argument($.number)
    ),

    _revoked_host_keys: $ => seq(
      keyword('RevokedHostKeys'),
      $._sep,
      argument($._file_string)
    ),

    _security_key_provider: $ => seq(
      keyword('SecurityKeyProvider'),
      $._sep,
      argument($._security_key_provider_arg)
    ),

    _security_key_provider_arg: $ => choice(
      $._string,
      $._var_value
    ),

    _send_env: $ => seq(
      keyword('SendEnv'),
      $._sep,
      list($._space, argument($._send_env_value))
    ),

    _send_env_value: $ => seq(
      optional('-'),
      alias(
        pattern(/[a-zA-Z0-9_]/),
        $.variable
      )
    ),

    _server_alive_count_max: $ => seq(
      keyword('ServerAliveCountMax'),
      $._sep,
      argument($.number)
    ),

    _server_alive_interval: $ => seq(
      keyword('ServerAliveInterval'),
      $._sep,
      argument($.number)
    ),

    _session_type: $ => seq(
      keyword('SessionType'),
      $._sep,
      argument(choice(
        'none', 'subsystem', 'default'
      ))
    ),

    _set_env: $ => seq(
      keyword('SetEnv'),
      $._sep,
      list($._space, argument($._set_env_value))
    ),

    _set_env_value: $ => seq(
      alias($._var_name, $.variable),
      '=',
      $._string
    ),

    _stdin_null: $ => seq(
      keyword('StdinNull'),
      $._sep,
      argument($._boolean)
    ),

    _stream_local_bind_mask: $ => seq(
      keyword('StreamLocalBindMask'),
      $._sep,
      argument(alias(/0?[0-7]{3}/, $.number))
    ),

    _stream_local_bind_unlink: $ => seq(
      keyword('StreamLocalBindUnlink'),
      $._sep,
      argument($._boolean)
    ),

    _strict_host_key_checking: $ => seq(
      keyword('StrictHostKeyChecking'),
      $._sep,
      argument($._strict_host_key_checking_arg)
    ),

    _strict_host_key_checking_arg: $ => choice(
      $._boolean,
      'accept-new',
      'off',
      'ask'
    ),

    _syslog_facility: $ => seq(
      keyword('SyslogFacility'),
      $._sep,
      argument($.facility)
    ),

    _tcp_keep_alive: $ => seq(
      keyword('TCPKeepAlive'),
      $._sep,
      argument($._boolean)
    ),

    _tag: $ => seq(
      keyword('Tag'),
      $._sep,
      argument($._string)
    ),

    _tunnel: $ => seq(
      keyword('Tunnel'),
      $._sep,
      argument($._tunnel_arg)
    ),

    _tunnel_arg: $ => choice(
      $._boolean,
      'point-to-point',
      'ethernet'
    ),

    _tunnel_device: $ => seq(
      keyword('TunnelDevice'),
      $._sep,
      argument($._tunnel_device_arg)
    ),

    _tunnel_device_arg: $ => seq(
      field('local_tun', choice('any', $.number)),
      optional(seq(
        ':',
        field('remote_tun', choice('any', $.number))
      ))
    ),

    _update_host_keys: $ => seq(
      keyword('UpdateHostKeys'),
      $._sep,
      argument(choice($._boolean, 'ask'))
    ),

    _use_keychain: $ => seq(
      keyword('UseKeychain'),
      $._sep,
      $._boolean
    ),

    _user: $ => seq(
      keyword('User'),
      $._sep,
      argument($._string)
    ),

    _user_known_hosts_file: $ => seq(
      keyword('UserKnownHostsFile'),
      $._sep,
      list($._space, argument(
        choice('none', $._file_string)
      ))
    ),

    _verify_host_key_dns: $ => seq(
      keyword('VerifyHostKeyDNS'),
      $._sep,
      argument(choice($._boolean, 'ask'))
    ),

    _visual_host_key: $ => seq(
      keyword('VisualHostKey'),
      $._sep,
      argument($._boolean)
    ),

    _xauth_location: $ => seq(
      keyword('XAuthLocation'),
      $._sep,
      argument($._string)
    ),

    ipqos: _ => token(choice(
      'af11', 'af12', 'af13',
      'af21', 'af22', 'af23',
      'af31', 'af32', 'af33',
      'af41', 'af42', 'af43',
      'cs0', 'cs1', 'cs2',
      'cs3', 'cs4', 'cs5',
      'cs6', 'cs7', 'ef',
      'le', 'lowdelay',
      'throughput',
      'reliability'
    )),

    verbosity: _ => token(choice(
      'QUIET', 'FATAL', 'ERROR', 'INFO', 'VERBOSE',
      'DEBUG', 'DEBUG1', 'DEBUG2', 'DEBUG3'
    )),

    facility: _ => token(choice(
      'DAEMON', 'USER', 'AUTH',
      'LOCAL0', 'LOCAL1', 'LOCAL2', 'LOCAL3',
      'LOCAL4', 'LOCAL5', 'LOCAL6', 'LOCAL7'
    )),

    authentication: _ => token(choice(
      'gssapi-with-mic', 'hostbased', 'publickey',
      'keyboard-interactive', 'password'
    )),

    // ssh -Q cipher
    cipher: _ => token(choice(
       '3des-cbc',
       'aes128-cbc',
       'aes192-cbc',
       'aes256-cbc',
       'aes128-ctr',
       'aes192-ctr',
       'aes256-ctr',
       'aes128-gcm@openssh.com',
       'aes256-gcm@openssh.com',
       'chacha20-poly1305@openssh.com',
    )),

    // ssh -Q kex
    kex: _ => token(choice(
      'diffie-hellman-group1-sha1',
      'diffie-hellman-group14-sha1',
      'diffie-hellman-group14-sha256',
      'diffie-hellman-group16-sha512',
      'diffie-hellman-group18-sha512',
      'diffie-hellman-group-exchange-sha1',
      'diffie-hellman-group-exchange-sha256',
      'ecdh-sha2-nistp256',
      'ecdh-sha2-nistp384',
      'ecdh-sha2-nistp521',
      'curve25519-sha256',
      'curve25519-sha256@libssh.org',
      'sntrup761x25519-sha512@openssh.com',
    )),

    // ssh -Q key-sig
    key_sig: _ => token(choice(
      'ssh-ed25519',
      'ssh-ed25519-cert-v01@openssh.com',
      'sk-ssh-ed25519@openssh.com',
      'sk-ssh-ed25519-cert-v01@openssh.com',
      'ecdsa-sha2-nistp256',
      'ecdsa-sha2-nistp256-cert-v01@openssh.com',
      'ecdsa-sha2-nistp384',
      'ecdsa-sha2-nistp384-cert-v01@openssh.com',
      'ecdsa-sha2-nistp521',
      'ecdsa-sha2-nistp521-cert-v01@openssh.com',
      'sk-ecdsa-sha2-nistp256@openssh.com',
      'sk-ecdsa-sha2-nistp256-cert-v01@openssh.com',
      'webauthn-sk-ecdsa-sha2-nistp256@openssh.com',
      'ssh-dss',
      'ssh-dss-cert-v01@openssh.com',
      'ssh-rsa',
      'ssh-rsa-cert-v01@openssh.com',
      'rsa-sha2-256',
      'rsa-sha2-256-cert-v01@openssh.com',
      'rsa-sha2-512',
      'rsa-sha2-512-cert-v01@openssh.com',
    )),

    // ssh -Q mac
    mac: _ => token(choice(
      'hmac-sha1',
      'hmac-sha1-96',
      'hmac-sha2-256',
      'hmac-sha2-512',
      'hmac-md5',
      'hmac-md5-96',
      'umac-64@openssh.com',
      'umac-128@openssh.com',
      'hmac-sha1-etm@openssh.com',
      'hmac-sha1-96-etm@openssh.com',
      'hmac-sha2-256-etm@openssh.com',
      'hmac-sha2-512-etm@openssh.com',
      'hmac-md5-etm@openssh.com',
      'hmac-md5-96-etm@openssh.com',
      'umac-64-etm@openssh.com',
      'umac-128-etm@openssh.com',
    )),

    // ssh -Q sig
    sig: _ => token(choice(
      'ssh-ed25519',
      'sk-ssh-ed25519@openssh.com',
      'ecdsa-sha2-nistp256',
      'ecdsa-sha2-nistp384',
      'ecdsa-sha2-nistp521',
      'sk-ecdsa-sha2-nistp256@openssh.com',
      'webauthn-sk-ecdsa-sha2-nistp256@openssh.com',
      'ssh-dss',
      'ssh-rsa',
      'rsa-sha2-256',
      'rsa-sha2-512',
    )),

    _file_token: $ => alias(/%[%CdhikLlnpru]/, $.token),

    _hosts_token: $ => alias(/%[%CdhikLlnprufHIKt]/, $.token),

    _hostname_token: $ => alias(/%[%h]/, $.token),

    _proxy_token: $ => alias(/%[%hnpr]/, $.token),

    token: _ => /%[%CdfHhIiKkLlnprTtu]/,

    _var_value: $ => alias(
      seq('$', field('name', $._var_name)),
      $.variable
    ),

    _var_name: _ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    variable: $ => seq('${', field('name', $._var_name), '}'),

    _file_string: $ => choice(
      alias(token_(/\S/, $._file_token, $.variable), $.string),
      seq('"', alias(token_($._char, $._file_token, $.variable), $.string), '"'),
    ),

    _hosts_string: $ => alias(
      token_(/[^\r\n]/, $._hosts_token, $.variable),
      $.string
    ),

    _hostname_string: $ => choice(
      alias(token_(/\S/, $._hostname_token), $.string),
      seq('"', alias(token_($._char, $._hostname_token), $.string), '"')
    ),

    _proxy_string: $ => alias(
      token_(/[^\r\n]/, $._proxy_token),
      $.string
    ),

    _token_string: $ => alias(
      token_(/[^\r\n]/, $.token),
      $.string
    ),

    _string: $ => choice(
      $._plain_string,
      seq('"', alias(repeat1($._char), $.string), '"')
    ),

    _plain_string: $ => alias(repeat1(/\S/), $.string),

    _file_pattern: $ => choice(
      alias(pattern(/\S/, $._file_token), $.pattern),
      seq('"', alias(pattern($._char, $._file_token), $.pattern), '"'),
    ),

    _file_pattern_vars: $ => choice(
      alias(pattern(/\S/, $.variable, $._file_token), $.pattern),
      seq('"', alias(pattern($._char, $.variable, $._file_token), $.pattern), '"')
    ),

    _pattern: $ => choice(
      alias(pattern(/\S/), $.pattern),
      seq('"', alias(pattern($._char), $.pattern), '"')
    ),

    _boolean: _ => choice('yes', 'no'),

    _number: _ => /[1-9][0-9]*|0/,

    number: $ => prec(1, $._number),

    bytes: $ => seq(
      $._number,
      optional(/[kmgKMG]/)
    ),

    time: $ => choice(
      $._number,
      seq($._number, /[wW]/),
      seq(
          optional(seq($._number, /[wW]/)),
          $._number, /[dD]/,
      ),
      seq(
          optional(seq($._number, /[wW]/)),
          optional(seq($._number, /[dD]/)),
          $._number, /[hH]/,
      ),
      seq(
          optional(seq($._number, /[wW]/)),
          optional(seq($._number, /[dD]/)),
          optional(seq($._number, /[hH]/)),
          $._number, /[mM]/,
      ),
      seq(
          optional(seq($._number, /[wW]/)),
          optional(seq($._number, /[dD]/)),
          optional(seq($._number, /[hH]/)),
          optional(seq($._number, /[mM]/)),
          $._number, /[sS]/,
      )
    ),

    comment: _ => /#.*/,

    _sep: $ => choice(
      $._space,
      alias(/[ \t]*=[ \t]*/, '=')
    ),

    _char: _ => /[^"]|\\"/,

    _space: _ => /[ \t]+/,

    _eol: _ => /\r?\n/
  }
});
