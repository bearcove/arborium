/*  Part of SWI-Prolog

    Author:        Jan Wielemaker
    E-mail:        J.Wielemaker@vu.nl
    WWW:           http://www.swi-prolog.org
    Copyright (c)  2019, VU University Amsterdam
    All rights reserved.

    Redistribution and use in source and binary forms, with or without
    modification, are permitted provided that the following conditions
    are met:

    1. Redistributions of source code must retain the above copyright
       notice, this list of conditions and the following disclaimer.

    2. Redistributions in binary form must reproduce the above copyright
       notice, this list of conditions and the following disclaimer in
       the documentation and/or other materials provided with the
       distribution.

    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
    "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
    LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS
    FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE
    COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
    INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING,
    BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
    LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
    CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
    LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN
    ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
    POSSIBILITY OF SUCH DAMAGE.
*/

:- module(http_server,
          [ http_server/1,		% +Options
            % from thread_httpd
            http_current_server/2,      % ?:Goal, ?Port
            http_server_property/2,     % ?Port, ?Property
            http_server/2,              % :Goal, +Options
            http_workers/2,             % +Port, ?WorkerCount
            http_add_worker/2,          % +Port, +Options
            http_current_worker/2,      % ?Port, ?ThreadID
            http_stop_server/2,         % +Port, +Options
            http_spawn/2,               % :Goal, +Options
            % from http_dispatch
            http_dispatch/1,            % +Request
            http_handler/3,             % +Path, +Predicate, +Options
            http_delete_handler/1,      % +Path
            http_request_expansion/2,   % :Goal, +Rank
            http_reply_file/3,          % +File, +Options, +Request
            http_redirect/3,            % +How, +Path, +Request
            http_404/2,                 % +Options, +Request
            http_switch_protocol/2,     % :Goal, +Options
            http_current_handler/2,     % ?Path, ?Pred
            http_current_handler/3,     % ?Path, ?Pred, -Options
            http_location_by_id/2,      % +ID, -Location
            http_link_to_id/3,          % +ID, +Parameters, -HREF
            http_reload_with_parameters/3, % +Request, +Parameters, -HREF
            % from http_wrapper
            http_current_request/1,     % -Request
            http_peer/2,
            % from http_parameters
            http_parameters/2,          % +Request, -Params
            http_parameters/3,
            % from html_write
            reply_html_page/2,          % :Head, :Body
            html//1,                    % :Content
                                        % Extension support
            (html_meta)/1,              % +Spec
            op(1150, fx, html_meta),
            % from http_json
            reply_json_dict/1,          % +JSON
            reply_json_dict/2,          % +JSON, Options
            http_read_json_dict/2,      % +Request, -Dict
            http_read_json_dict/3,      % +Request, -Dict, +Options

            is_json_content_type/1
          ]).
:- use_module(library(http/thread_httpd)).
:- use_module(library(http/http_dispatch)).
:- use_module(library(http/http_wrapper)).
:- use_module(library(http/http_parameters)).
:- use_module(library(http/html_write)).
:- use_module(library(http/http_json)).
:- use_module(library(http/http_dyn_workers)).

/** <module> HTTP server library

This  library  combines  the  core  server  functionality  provided  by
several libraries that are needed by almost any web server.  It exports
the   commonly   used  predicates   from    library(http/thread_httpd),
library(http/http_dispatch),                library(http/http_wrapper),
library(http/http_parameters),                library(http/html_write),
library(http/http_json),      and       library(http/http_dyn_workers).

*/

%!  http_server(+Options) is det.
%
%   Create an HTTP server using   http_dispatch/1 for handling requests.
%   See http_server/2 and http_dispatch/1 for details.

http_server(Options) :-
    http_server(http_dispatch, Options).
