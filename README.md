# Stale

```stale``` is a small utility tool written in Rust designed to detect when stdout is stale.

## Usage
```bash
tail -f myapp.log | stale --delay 20 -m "Alert: stdout is stale since [{staletime}]"
```

This will print "Alert: Stdout is stale" if no line is printed to stdout for 20 seconds.
```
Alert: stdout is stale
```

## Options

### Delay
```bash
--delay n or -d n
```
Where n is the delay expressed in seconds.

### Message
```bash
--message "..." or -m "..."
```
To provide a custom message when stale stream is detected.
A few substitution strings are available :
- "{now}" : will be replaced by current local time
- "{staletime}" : will be replaced by the local time at which the latest event was detected.

### Passthrough
```bash
--passthrough or -p
```
Will echo lines piped to stdout. By default, ```stale``` is NOT printing anything, excepted the alert message when a detection occurs

### Norearm
```bash
--norearm or -n
```
Alert will be triggered only once, else it will be triggered every <delay> until an event is seen.
