function fisher --argument-names cmd --description "A plugin manager for Fish"
    set --query fisher_path || set --local fisher_path $__fish_config_dir
    set --local fisher_version 4.4.5
    set --local fish_plugins $__fish_config_dir/fish_plugins

    switch "$cmd"
        case -v --version
            echo "fisher, version $fisher_version"
        case "" -h --help
            echo "Usage: fisher install <plugins...>  Install plugins"
            echo "       fisher remove  <plugins...>  Remove installed plugins"
            echo "       fisher update  <plugins...>  Update installed plugins"
            echo "       fisher update                Update all installed plugins"
            echo "       fisher list    [<regex>]     List installed plugins matching regex"
            echo "Options:"
            echo "       -v, --version  Print version"
            echo "       -h, --help     Print this help message"
            echo "Variables:"
            echo "       \$fisher_path  Plugin installation path. Default: $__fish_config_dir" | string replace --regex -- $HOME \~
        case ls list
            string match --entire --regex -- "$argv[2]" $_fisher_plugins
        case install update remove
            isatty || read --local --null --array stdin && set --append argv $stdin

            set --local install_plugins
            set --local update_plugins
            set --local remove_plugins
            set --local arg_plugins $argv[2..-1]
            set --local old_plugins $_fisher_plugins
            set --local new_plugins

            test -e $fish_plugins && set --local file_plugins (string match --regex -- '^[^\s]+$' <$fish_plugins | string replace -- \~ ~)

            if ! set --query argv[2]
                if test "$cmd" != update
                    echo "fisher: Not enough arguments for command: \"$cmd\"" >&2 && return 1
                else if ! set --query file_plugins
                    echo "fisher: \"$fish_plugins\" file not found: \"$cmd\"" >&2 && return 1
                end
                set arg_plugins $file_plugins
            end

            for plugin in $arg_plugins
                set plugin (test -e "$plugin" && realpath $plugin || string lower -- $plugin)
                contains -- "$plugin" $new_plugins || set --append new_plugins $plugin
            end

            if set --query argv[2]
                for plugin in $new_plugins
                    if contains -- "$plugin" $old_plugins
                        test "$cmd" = remove &&
                            set --append remove_plugins $plugin ||
                            set --append update_plugins $plugin
                    else if test "$cmd" = install
                        set --append install_plugins $plugin
                    else
                        echo "fisher: Plugin not installed: \"$plugin\"" >&2 && return 1
                    end
                end
            else
                for plugin in $new_plugins
                    contains -- "$plugin" $old_plugins &&
                        set --append update_plugins $plugin ||
                        set --append install_plugins $plugin
                end

                for plugin in $old_plugins
                    contains -- "$plugin" $new_plugins || set --append remove_plugins $plugin
                end
            end

            set --local pid_list
            set --local source_plugins
            set --local fetch_plugins $update_plugins $install_plugins
            set --local fish_path (status fish-path)

            echo (set_color --bold)fisher $cmd version $fisher_version(set_color normal)

            for plugin in $fetch_plugins
                set --local source (command mktemp -d)
                set --append source_plugins $source

                command mkdir -p $source/{completions,conf.d,themes,functions}

                $fish_path --command "
                    if test -e $plugin
                        command cp -Rf $plugin/* $source
                    else
                        set temp (command mktemp -d)
                        set repo (string split -- \@ $plugin) || set repo[2] HEAD

                        if set path (string replace --regex -- '^(https://)?gitlab.com/' '' \$repo[1])
                            set name (string split -- / \$path)[-1]
                            set url https://gitlab.com/\$path/-/archive/\$repo[2]/\$name-\$repo[2].tar.gz
                        else
                            set url https://api.github.com/repos/\$repo[1]/tarball/\$repo[2]
                        end

                        echo Fetching (set_color --underline)\$url(set_color normal)

