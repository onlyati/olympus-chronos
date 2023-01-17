# Client usage

The `--help` option of client describes what you can do:
```
Usage: chronos-cli [OPTIONS] --hostname <HOSTNAME> <COMMAND>

Commands:
  verbose-log-on   Turn on verbose log
  verbose-log-off  Turn off verbose log
  list-active      List currently active timers
  list-static      List static timers
  purge            Purge active timer
  create           Create dynamic timer
  refresh          Refresh static timer
  help             Print this message or the help of the given subcommand(s)

Options:
  -H, --hostname <HOSTNAME>  Specifiy the host name or config pointer, for example: http://example.com or cfg://example
  -c, --config <CONFIG>      If cfg:// specified at hostname, then this is where the config is read [default: /etc/olympus/chronos/client.conf]
  -v, --verbose              Show more detail about connection
  -h, --help                 Print help
```

If secured connection is used, then connection information must be provided by sugin config file. This can be define with th `-c` option. If not defined, default is trying to be used: `/etc/olympus/chronos/client.conf`.
This config file can contain more server's host information. For example:
```
node.server1.address = https://server1.lan:9150
node.server1.ca_cert = /etc/olympus/chronos/certs/hepha_pr_ca.pem
node.server1.domain = server1.lan

node.server2.address = https://server1.lan:9150
node.server2.ca_cert = /etc/olympus/chronos/certs/hepha_pr_ca.pem
node.server2.domain = server1.lan
```

when `-H cfg://server1` or `-H cfg://server2` option is used, then connection information will be read from here.
