# Telemetry Policy

This fork does not collect or transmit analytics.

The policy is enforced in code and cannot be enabled with a command-line flag,
configuration file, or environment variable. It covers usage events, feedback,
session transcripts, installation attribution, and sponsor usage metering.

At startup, Jcode removes legacy telemetry state created by earlier builds.
`jcode telemetry status` reports the permanent fork policy; the `enable`,
and `disable` subcommands cannot change it.

Features that make an explicit network request at the user's direction are not
analytics. In particular, `integration_tools` and its `discover_tools` alias
remain available when invoked.
