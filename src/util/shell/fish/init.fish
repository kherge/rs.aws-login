###
# Integrates the AWS Login command with the shell.
#
# This function will be invoked instead of the command whenever you type
# `{AWS_LOGIN}` and is responsible for evaluating any shell code it writes
# to a file.
##
function aws-login
    # Create the shell script file.
    if set -x AWS_LOGIN_SCRIPT (mktemp)

        # Execute the real command.
        {AWS_LOGIN_SHELL}=bash "{AWS_LOGIN}" $argv

        set STATUS $status

        # Evaluate the shell script if it is not empty.
        if test -s "$AWS_LOGIN_SCRIPT"
            cat "$AWS_LOGIN_SCRIPT" | source

            rm "$AWS_LOGIN_SCRIPT"

            return $STATUS
        end

        return 1
    end
end
