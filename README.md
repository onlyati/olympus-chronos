# Chronos@Olympus

Chronos is a timer module in Olympus application package. Task of Chronos are the following: execute timed actions based on timer rules.
Similar like `cron` but with more flexibility.

## Current abilities

Chronos is able to do the following activities:
- Tasks can be created by using files (see in `sample/all_timers`)
- Timer tasks can be activated automatically after Chronos tart (having symlink in `startup_timers`) or activating via CLI
- Timer tasks can be executed by any operator which is defined in task
- There are 3 different timer type:
  - Every: Running every specified interval
  - At: Running once a day in the specified time
  - Oneshot: Running once after activation when interval has passed

## Plans for the future

Following functions are planned for the future:
- Support weekday attribute in timer task file
- Implement cluster feature

## Installation

It is really just a hobby project and in a **very early stage**, but if you would like to try it, you can do by the following steps.

1. Clone the repository
2. Copy `olmypus.chronos.service` file into `/lib/systemd/system` directory (or anywhere where you like to prefer systemd service files)
3. Refresh systemd: `systemctl daemon-reload`
4. Be assumed that `/usr/share/olympus/chronos` and `/etc/olympus/chronos` exists
5. Create a new group, called `olympus` and assign yourself for it
6. In a terminal, navigate to the cloned repository's directory and execute `make publish` command
7. Create a symlink for `/usr/share/olympus/chronos/cli` in `/usr/local/bin` with `chronos-cli` name: `cd /usr/local/bin/ && sudo ln -s /usr/share/olympus/chronos/cli chronos-cli`

## Usage
By checking `chronos-cli` help, you can see what you can do:

```
❯ chronos-cli help
Possible Chronos commands:
List active timers:                  list active
List started timers:                 list startup
List details of started timers:      list startup expanded
List all timer config:               list all
List details of all timer config:    list all expanded
Purge timer:                         purge <timer-id>
Add timer:                           add <timer-id>
Enable startup timer:                startup enable <timer-id>
Disable startup timer:               startup disable <timer-id>
```

