AWS Login
=========

A command line utility to simplify logging into AWS accounts and services.

- [Requirements](#requirements)
    - [Development](#development)
- [Installation](#installation)
    - [macOS](#macos)
    - [Integration into Shell](#integration-into-shell)
- [Usage](#usage)
    - [Configuring Docker to use ECR](#configuring-docker-to-use-ecr)
    - [Configure `kubectl` to use EKS](#configure-kubectl-to-use-eks)
    - [Log into an AWS account using SSO](#log-into-an-aws-account-using-sso)
    - [Choosing a profile](#choosing-a-profile)
        - [Working with profile templates](#working-with-profile-templates)
    - [Downloading and installing profile templates](#downloading-and-installing-profile-templates)

Requirements
------------

- AWS CLI 2

### Development

- Rust 1.57+

Installation
------------

1. Go to the [Releases] page.
2. Find the latest release.
3. Download the correct ZIP file for your OS.
4. Unzip the file.
5. Move the `aws-login` from the extracted `target/release/` folder to somewhere in your `$PATH`.
6. Make `aws-login` executable (e.g. `chmod 755 aws-login` on Linux and macOS).

### macOS

On more recent versions of macOS, Gatekeeper will block your attempt to run the application because it is not signed with an Apple Developer certificate. I have no intention of paying for the fee any time soon, so you can either build the application yourself from source or follow this guide:

1. Run the application.
2. Click **Cancel** in this dialog.<br/>![Scary Dialog #1](assets/1.png)
3. Go to **System Preferences**.<br/>![System Preferences](assets/2.png)
4. Go to **Security & Privacy**.
5. You should see a message stating that `"aws-login" was blocked from use because it is not from an identified developer.` Click **Allow Anyway**.<br/>![Security & Privacy](assets/3.png)
5. Run the application again.
6. Click **Open** in this dialog.<br/>![Scary Dialog #2](assets/4.png)
7. (Optional) Submit [feedback to Apple] to support code signing for FOSS developers.

You only have to do this once per installation or update.

[Releases]: https://github.com/kherge/rs.aws-login/releases/latest
[feedback to Apple]: https://www.apple.com/feedback/macos.html

### Integration into Shell

The `aws-login` utility is better when it is integrated into your shell environment. This allows the utility to make temporary changes that will improve your experience while using the AWS CLI. This integration is entirely optional and the utility will fallback to providing you instructions on how to do things manually.

1. Install `aws-login` per the instructions above.
2. Run `aws-login install --shell <SHELL>`.

You will need to replace `<SHELL>` with the name of the shell you are actively using. The following shells are currently supported:

- `bash` -- Integreates into the Bourne Again SHell (most common choice).
- `posix` -- Integrates into any POSIX compliant shell (fallback).
- `zsh` -- Integrates into Z Shell (default on modern macOS).

The `posix` fallback is less seamless but works identically to the other options. Instead of using the `aws-login` command, however, you will need to use `aws_login` since POSIX-only shells do not support the use of hyphens in function names. You can look at the [installer] if you're curious on how this works.

[installer]: src/app/subcommand/install/installer.rs

Usage
-----

There are a few things you can do with `aws-login`.

### Configuring Docker to use ECR

    aws-login ecr

This command will configure Docker to use the Elastic Container Registry in the account for your active AWS CLI profile. If the region for your ECR differs from the default region configured for your profile, remember to specify it with the `--region` option.

### Configure `kubectl` to use EKS

    aws-login eks

This command will prompt you to choose an EKS cluster from a list found in the account for your active AWS CLI profile. Once a selection is made, the configuration for `kubectl` is updated to support connecting to that EKS cluster. Remember to log in before attempting to do so, fresh credentials may be required.

### Log into an AWS account using SSO

    aws-login sso

This command will attempt to log you into the account for your active AWS CLI profile. If the SSO configuration for the profile is incomplete, you will be prompted to provide additional configuration setting values before SSO authentication can proceed.

### Choosing a profile

    aws-login pick

This command will prompt you to choose a profile from a list of existing AWS CLI profiles. Once a profile has been selected, the utility will attempt to set the `AWS_PROFILE` environment variable for you. This will allow you to use the `aws` CLI without having to specify `--profile` for each execution.

This command will also prompt you to choose a profile from a list of profile templates. If an AWS CLI profile of the same name does not already exist, the profile will be created for you using the template.

#### Working with profile templates

In essence, profile templates are named sets of profile configuration settings.

```json
{
    "base": {
        "enabled": false,
        "settings": {
            "output": "json",
            "region": "us-east-1",
            "sso_region": "us-east-1",
            "sso_start_url": "https://my-sso-portal.awsapps.com/start"
        }
    },
    "dev-read": {
        "extends": "base",
        "settings": {
            "sso_account_id": 123456789012,
            "sso_role_name": "ReadOnly"
        }
    },
    "dev-write": {
        "extends": "base",
        "settings": {
            "sso_account_id": 123456789012,
            "sso_role_name": "Developer"
        }
    }
}
```

> This file may be found in `$HOME/.config/aws-login/templates.json` (or `%APPDATA%\AWS Login\templates.json` in Windows). If it does not exist, you will need to create a new or copy an existing one.

The `base` template has `enabled` set to `false` so that it is not presented as an option when you use `aws-login pick` to select a profile. The template is disabled because it is intended to be use as the base for the profile templates `dev-read` and `dev-write`. More will be explain with the other profiles.

The `dev-read` template `extends` the `base` template in order to inherit all of its configuration settings. If `dev-read` and `base` both define the same setting, `dev-read` would override `base` during inheritance.

The `dev-write` profile template is very similar to `dev-read`, except that a different role is used for that profile.

As you can see, we have some flexibility with how we define our profile templates. We can define profile settings we commonly use in all of our profiles in one location, and allow other profiles to make adjustments as needed.

These templates are intended to be shared with colleagues in an organization where everyone generally uses the same set of profile configuration settings. With that in mind, it is not advised that any credentials or other sensitive settings be kept in the profile template. Sensitive settings should be defined directly in the AWS CLI profile using `aws --profile <PROFILE> configure set <VALUE>`.

### Downloading and installing profile templates

    aws-login pull https://www.example.com/path/to/templates.json

This command will download a remote profile templates file and store a copy for later use. If a local templates file already exists, you will be asked if you would like to replace it. The subcommand does not currently handle merging of templates.
