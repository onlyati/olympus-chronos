# Configuration

When Chronos is starting, config file has to be provided as parameter of program. Sample configuration file:
```conf
*
* Host addresses
*
host.grpc.address = localhost:3042               // gRPC host address
host.grpc.tls = yes                              // yes or no to enable/disable tls
host.grpc.tls.key = /home/ati/work/OnlyAti.Chronos/other/certs/chronos_test.key
host.grpc.tls.pem = /home/ati/work/OnlyAti.Chronos/other/certs/chronos_test.pem

*
* Timer related settings
*
timer.all_dir = /home/ati/work/OnlyAti.Chronos/other/all_timers
timer.log_dir = /home/ati/work/OnlyAti.Chronos/other/logs

*
* Fill these to allow escalate statuses to Hermes
*
hermes.enable = yes
hermes.grpc.address = http://atihome.lan:9099     // gRPC address and port of Hermes
hermes.grpc.tls = no
hermes.grpc.tls.ca_cert = /placeholder
hermes.grpc.tls.domain = placeholder
hermes.table = ChronosTest                        // Which table should the records send
hermes.key.prefix = timer/atihome/                // Prefix for key value in Hermes, "/<timer-id>" is added

*
* Other settings
*
defaults.verbose = no                            // Verbose output is required by default?
```

If everything is fine, output looks like after start:
```
Version v.0.2.0 is starting...
Configuration:
- hermes.grpc.tls -> no
- host.grpc.tls.key -> /home/ati/work/OnlyAti.Chronos/other/certs/chronos_test.key
- host.grpc.address -> localhost:3042
- defaults.verbose -> no
- hermes.grpc.tls.domain -> placeholder
- host.grpc.tls.pem -> /home/ati/work/OnlyAti.Chronos/other/certs/chronos_test.pem
- hermes.key.prefix -> timer/atihome/
- hermes.table -> ChronosTest
- timer.all_dir -> /home/ati/work/OnlyAti.Chronos/other/all_timers
- host.grpc.tls -> yes
- hermes.enable -> yes
- hermes.grpc.address -> http://atihome.lan:9099
- hermes.grpc.tls.ca_cert -> /placeholder
- timer.log_dir -> /home/ati/work/OnlyAti.Chronos/other/logs
Corresponse properties are set to yes, so start Hermes client
Start gRPC endpoint in on 127.0.0.1:3042 with TLS
Hermes client is ready
```
