
# Reminders

This is a simple application that parses a text file for reminder
strings and compares the reminder's specified date/time against the
current time. If they match then a notification is sent with the
reminder's text as the message. It is expected to be executed
periodically via a system tool like cron.

Current notification channels are:
- print to stdout
- [Pushover](https://pushover.net)

## Reminder Specification

A reminder string is specified as follows:

```
/remind <date_time> <text>
/rem <date_time> <text>
/r <date_time> <text>
```

Additionally, markdown tasks can also be used:

```
- [ ] <text> /remind <date_time>
- [ ] <text> /rem <date_time>
- [ ] <text> /r <date_time>
```

The `<date_time>` format can be any of the following:

```
<MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM><am|pm> - Date @ <time>
<MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM>        - Date @ <time> (military)
<MM>/<DD>[/<YY>|/<YYYY>] <HH><am|pm>      - Date @ <time> (min=0)
<MM>/<DD>[/<YY>|/<YYYY>]                  - Date @ 8:00am

<sun|mon|tue|wed|thu|fri|sat> <HH>:<MM><am|pm> - <weekday> @ <time>
<sun|mon|tue|wed|thu|fri|sat> <HH>:<MM>        - <weekday> @ <time> (military)
<sun|mon|tue|wed|thu|fri|sat> <HH><am|pm>      - <weekday> @ <time> (min=0)
<sun|mon|tue|wed|thu|fri|sat>                  - <weekday> @ 8:00am

<HH>:<MM><am|pm> - Every day @ <time>
<HH>:<MM>        - Every day @ <time> (military)
<HH><am|pm>      - Every day @ <time> (min=0)

monthly  - Every month on the 1st @ 8:00am
biweekly - Every other Monday @ 8:00am (even weeks)
weekly   - Every Monday @ 8:00am
daily    - Every day @ 8:00am
```

Example reminders:
```
/remind 4/29/2020 11:00am test with date and time
/remind 11am test with time
/remind tue 10:00pm test on tuesday
/remind weekly test weekly
```

## Installation

Requires [Rust](https://www.rust-lang.org/).

```
% git clone https://github.com/insanum/reminders
% cd reminders
% cargo install --path .
```

The application binary will be located at `$HOME/.cargo/bin/reminders`.

## Usage

```
% $HOME/.cargo/bin/reminders -h
Usage: reminders [options]

Options:
    -c <file.yaml>      config file
    -t <timestamp>      time override '<YYYY/MM/DD HH:MM>'
    -p, --pushover      send test message to pushover
    -h, --help          print this help menu
```

The `-c` option is required.

Use `-t` to override the current time that is checked against. This is
useful for testing.

## Configuration

The YAML configuration file can contain the following variables:

- `file: <text_file>` - The text file to parse looking for reminder strings.
  If the file is prefixed with `http://` or `https://` then the file is
  fetched via HTTP, else the file is read from local disk. If this variable
  is missing then it's an error unless a `reminders:` variable exists.

- `http_auth: basic` - If present then Basic HTTP auth is performed when
  fetching the text file.

- `http_username: <username>` - The user name to use for Basic HTTP auth.

- `http_password: <password>` - The password to use for Basic HTTP auth.

- `pushover_app_token: <app_token>` - The Pushover application token to
  use when sending the notification to Pushover. If this variable is missing
  then the notification is printed to stdout.

- `pushover_user_key: <user_key>` - The Pushover user key to use when
  sending the notification to Pushover. If this variable is missing then
  the notification is printed to stdout.

- `reminders: <reminders...>` - Instead of specifying the `file:`, the
  reminder strings can simply be placed in the config file. Note that
  reminders found in both this variable and from `file:` are processed.
  See `test.yaml` for an example.

