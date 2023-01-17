# Chronos@Olympus

## :earth_africa: What is Olympus?

Olympus is name of my package which is intended to supervise a Linux server environment and provide its applications and services a stable backend. Olympus consist of:
- **Zeus:** Responsible to run defined applications on proper server machines
- **Hermes:** Act like an in-memory database and a message queue for other Olympus applications
- **Chronos:** Execute commands by timely manner
- **Hephaestus:** Run long and complex tasks in the background as jobs
- **Apollo:** Center of documentation, stores every information and thresholds for monitoring scripts
- **Argos:** Collecting and analyzing data and forward it to Athena
- **Athena:** Automation of Olympus, it analyzes what other component does and act according to its rules

## :alarm_clock: Structure Chronos

Chronos is a timer module. It is made to execute commands, with short execution time, at timely manner.
Timer can be scheduled on two ways:
- Static timers: This is created from timer directory's file. This is scheduled automatically during Chronos startup.
- Dynamic timers: This is created by request. Chronos hosting a gRPC interface and it has an endpoint to create and schedule dynmic timer

Chronos has a CLI too, this help explain what can be done via this:
```
Usage: cli [OPTIONS] --hostname <HOSTNAME> <COMMAND>

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