#!/bin/bash

###
# Integrates the AWS Login command with the shell.
#
# This function will be invoked instead of the command whenever you type
# `{AWS_LOGIN}` and is responsible for evaluating any shell code it writes
# to a file.
#
# shellcheck disable=SC2288
# shellcheck disable=SC3033
# shellcheck disable=SC3043
##
aws-login()
{
    # Create the shell script file.
    local AWS_LOGIN_SCRIPT=

    if AWS_LOGIN_SCRIPT="$(mktemp)"; then

        # Execute the real command.
        AWS_LOGIN_SCRIPT="$AWS_LOGIN_SCRIPT" {AWS_LOGIN_SHELL}=bash "{AWS_LOGIN}" "$@"

        local STATUS=$?

        # Evaluate the shell script if it is not empty.
        if [ -s "$AWS_LOGIN_SCRIPT" ]; then
            eval "$(cat "$AWS_LOGIN_SCRIPT")"
        fi

        rm "$AWS_LOGIN_SCRIPT"

        return $STATUS
    fi

    return 1
}
