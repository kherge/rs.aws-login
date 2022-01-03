###
# Integrates the AWS Login command with the shell.
#
# This function will be invoked instead of the command whenever you type
# `{AWS_LOGIN}` and is responsible for evaluating any shell code it writes
# to a file.
##
function aws-login {
    # Create the shell script file.
    $AwsLoginScript = New-TemporaryFile

    # Execute the real command.
    $Env:AWS_LOGIN_SCRIPT = $AwsLoginScript
    $Env:{AWS_LOGIN_SHELL} = 'powershell'

    {AWS_LOGIN} @args

    Remove-Item Env:AWS_LOGIN_SCRIPT
    Remove-Item Env:{AWS_LOGIN_SHELL}

    # Evaluate the shell script if it is not empty.
    if ($Contents = Get-Content $AwsLoginScript -Raw) {
        $null = Invoke-Expression $Contents
    }

    Remove-Item $AwsLoginScript
}
