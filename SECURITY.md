# Security Policy

## Supported versions

Only the latest release receives security fixes. The site runs the latest release, so a fix ships
as a new release rather than as a backport.

| Version        | Supported |
| -------------- | --------- |
| Latest release | Yes       |
| Older releases | No        |

## Reporting a vulnerability

Do not open a public issue for a vulnerability.

Report it privately through
[GitHub Security Advisories](https://github.com/TimSchoenle/Portfolio/security/advisories/new).
If you cannot use GitHub, email <contact@tim-schoenle.de> with "Security" in the subject.

Include the affected version or commit, the steps to reproduce, and the impact you observed. You
can expect an acknowledgement within seven days. Please allow a fix to be released before
disclosing the issue publicly.

## Scope

In scope: this repository's code, the container image it publishes, and the site it serves at
<https://tim-schoenle.de>. Findings that require a compromised Cloudflare account, a compromised
host, or physical access to a visitor's device are out of scope.
