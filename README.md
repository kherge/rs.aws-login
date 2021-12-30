AWS Login
=========

A command line utility to simplify logging into AWS accounts and services.

Requirements
------------

- AWS CLI 2

### Development

- Rust 1.57+

Installation
------------

1. Go to the [Releases] page.
2. Find the latest release.
3. Download the asset correct for your OS.
4. Unzip the file.
5. Move the `aws-login` file in `target/release/` to somewhere in your `$PATH`.
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

[Releases]: https://github.com/kherge/rs.aws-login/releases
[feedback to Apple]: https://www.apple.com/feedback/macos.html

Usage
-----

There are a few things you can do with `aws-login`.

### Configuring Docker to use ECR

    aws-login ecr

This command will configure Docker to use the Elastic Container Registry in the account for your active AWS CLI profile. If the region for your ECR differs from the default configured for your profile, remember to specify it with the `--region` option.

### Configure `kubect` to use EKS

    aws-login eks

This command will prompt you to choose from a cluster from a list found in the account for your active AWS CLI profile. Once a selection is made, the configuration for `kubectl` is updated to support connecting to that EKS cluster. Remember to log in before attempting to do so, fresh credentials may be required.

### Log into an AWS account using SSO

    aws-login sso

This command will attempt to log you into the account for your active AWS CLI profile. If the SSO configuration for the profile is incomplete, you will be prompted to provide additional configuration setting values before SSO authentication can proceed.

### Choosing a profile

TBD
