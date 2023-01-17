# Timer file

During Chronos startup, timer defintions are read from timer files. Location of timer files are set in config by `timer.all_dir` property.

Possible timer file properties:
- type: Type of timer declares when it can run and how
  - Every: Timer would run after every interval has expired
  - At: Timer will run once a day. In this case the interval parameter tells when
  - Oneshot: After timer is activated (statically or dynamically) timer will run once after the interval has expired
- interval: How frequent or when timer should run
  - Must be in HH:MM:SS format
- command: What command should be executed by timer
  - If command should run other timer's name, then use sudo command: `sudo -u <user> <command>` (in this case chronos has to be run by a sudoer user)
- days: Which day timer should run
  - If this settings is omitted, then timer would run on each day
  - If specified, then it must be 7 charactrer length and contains only 'X' and '_' charcters. 'X' represent run, '_' represents does not run

## Sample timer files
```conf
type = every
interval = 00:00:30                // Timer would run in every 30 seconds
command = /usr/bin/script1.py      // This command would be executed
days = __X____                     // Run only at Wednesday
```

```conf
type = at                          // Run at 07:00:00 on every day
interval = 07:00:00
command = /usr/local/bin/hephaestus-cli -H cfg://atihome --plan-set backups --plan-name gitlab_backup exec
```

