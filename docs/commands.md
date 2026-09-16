# Command reference

Generated from the CLI command tree. All commands support --help.

Global options: --json, --plain, --yes, --experimental.

## mac status

Show a system overview

    mac status [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac menu

Browse and run commands interactively

    mac menu [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac commands

List commands and capability information

    mac commands [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac doctor

Check platform, permissions, and hardware without requesting access

    mac doctor [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac battery

Show battery charge, health, and power source

    mac battery [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac brightness

Control display brightness

    mac brightness [OPTIONS] [percent]

Requirements: Compatible display; private brightness API.

## mac brightness get

Read the current percentage

    mac brightness get [OPTIONS]

Requirements: Compatible display; private brightness API.

## mac brightness set

Set a percentage from 0 to 100

    mac brightness set [OPTIONS] <percent>

Requirements: Compatible display; private brightness API.

## mac brightness up

Increase by percentage points

    mac brightness up [OPTIONS] [step]

Requirements: Compatible display; private brightness API.

## mac brightness down

Decrease by percentage points

    mac brightness down [OPTIONS] [step]

Requirements: Compatible display; private brightness API.

## mac display list

List connected displays

    mac display list [OPTIONS]

Requirements: Compatible display; private brightness API.

## mac display modes

List modes for a display

    mac display modes [OPTIONS] <id>

Requirements: Compatible display; private brightness API.

## mac display set

Set a listed mode on a display

    mac display set [OPTIONS] <id> <mode>

Requirements: Compatible display; private brightness API.

## mac volume

Control output volume

    mac volume [OPTIONS] [percent]

Requirements: macOS 14+ on Apple Silicon.

## mac volume get

Read the current percentage

    mac volume get [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac volume set

Set a percentage from 0 to 100

    mac volume set [OPTIONS] <percent>

Requirements: macOS 14+ on Apple Silicon.

## mac volume up

Increase by percentage points

    mac volume up [OPTIONS] [step]

Requirements: macOS 14+ on Apple Silicon.

## mac volume down

Decrease by percentage points

    mac volume down [OPTIONS] [step]

Requirements: macOS 14+ on Apple Silicon.

## mac volume mute

Change the mute state

    mac volume mute [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac volume unmute

Change the mute state

    mac volume unmute [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac volume toggle

Change the mute state

    mac volume toggle [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac audio output list

List output devices

    mac audio output list [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac audio output set

Select an output device by ID

    mac audio output set [OPTIONS] <id>

Requirements: macOS 14+ on Apple Silicon.

## mac audio input list

List input devices

    mac audio input list [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac audio input set

Select an input device by ID

    mac audio input set [OPTIONS] <id>

Requirements: macOS 14+ on Apple Silicon.

## mac audio gain

Control input gain where supported

    mac audio gain [OPTIONS] [percent]

Requirements: macOS 14+ on Apple Silicon.

## mac audio gain get

Read the current percentage

    mac audio gain get [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac audio gain set

Set a percentage from 0 to 100

    mac audio gain set [OPTIONS] <percent>

Requirements: macOS 14+ on Apple Silicon.

## mac audio gain up

Increase by percentage points

    mac audio gain up [OPTIONS] [step]

Requirements: macOS 14+ on Apple Silicon.

## mac audio gain down

Decrease by percentage points

    mac audio gain down [OPTIONS] [step]

Requirements: macOS 14+ on Apple Silicon.

## mac keyboard brightness

Control keyboard backlight

    mac keyboard brightness [OPTIONS] [percent]

Requirements: Backlit keyboard or selectable input source.

## mac keyboard brightness get

Read the current percentage

    mac keyboard brightness get [OPTIONS]

Requirements: Backlit keyboard or selectable input source.

## mac keyboard brightness set

Set a percentage from 0 to 100

    mac keyboard brightness set [OPTIONS] <percent>

Requirements: Backlit keyboard or selectable input source.

## mac keyboard brightness up

Increase by percentage points

    mac keyboard brightness up [OPTIONS] [step]

Requirements: Backlit keyboard or selectable input source.

## mac keyboard brightness down

Decrease by percentage points

    mac keyboard brightness down [OPTIONS] [step]

Requirements: Backlit keyboard or selectable input source.

## mac keyboard source list

List selectable input sources

    mac keyboard source list [OPTIONS]

Requirements: Backlit keyboard or selectable input source.

## mac keyboard source set

Select an input source by ID

    mac keyboard source set [OPTIONS] <id>

Requirements: Backlit keyboard or selectable input source.

## mac wifi status

Show Wi-Fi status

    mac wifi status [OPTIONS]

Requirements: Wi-Fi hardware; Location Services for network names.

## mac wifi on

Enable Wi-Fi

    mac wifi on [OPTIONS]

Requirements: Wi-Fi hardware; Location Services for network names.

## mac wifi off

Disable Wi-Fi

    mac wifi off [OPTIONS]

Requirements: Wi-Fi hardware; Location Services for network names.

## mac wifi scan

Scan nearby networks; may require Location Services

    mac wifi scan [OPTIONS]

Requirements: Wi-Fi hardware; Location Services for network names.

## mac wifi connect

Connect to a Wi-Fi network; securely prompt for its password

    mac wifi connect [OPTIONS] <ssid>

Requirements: Wi-Fi hardware; Location Services for network names.

## mac bluetooth status

Read Bluetooth power

    mac bluetooth status [OPTIONS]

Requirements: Bluetooth hardware and permission.

## mac bluetooth on

Enable Bluetooth

    mac bluetooth on [OPTIONS]

Requirements: Bluetooth hardware and permission.

## mac bluetooth off

Disable Bluetooth

    mac bluetooth off [OPTIONS]

Requirements: Bluetooth hardware and permission.

## mac bluetooth list

List paired devices

    mac bluetooth list [OPTIONS]

Requirements: Bluetooth hardware and permission.

## mac bluetooth scan

Discover nearby Bluetooth devices

    mac bluetooth scan [OPTIONS]

Requirements: Bluetooth hardware and permission.

## mac bluetooth connect

Connect a paired device by address

    mac bluetooth connect [OPTIONS] <address>

Requirements: Bluetooth hardware and permission.

## mac bluetooth disconnect

Disconnect a device by address

    mac bluetooth disconnect [OPTIONS] <address>

Requirements: Bluetooth hardware and permission.

## mac network ip

List local interface addresses

    mac network ip [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac network interfaces

List network interfaces

    mac network interfaces [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac network ports

List listening TCP ports

    mac network ports [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac network dns get

Read DNS servers for a network service

    mac network dns get [OPTIONS] <service>

Requirements: macOS 14+ on Apple Silicon.

## mac network dns set

Set DNS servers for a network service

    mac network dns set [OPTIONS] <service> <servers>...

Requirements: macOS 14+ on Apple Silicon.

## mac network dns reset

Return a network service to automatic DNS

    mac network dns reset [OPTIONS] <service>

Requirements: macOS 14+ on Apple Silicon.

## mac network dns flush

Flush the system DNS cache

    mac network dns flush [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac power status

Read power settings

    mac power status [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac power low-power get

Read the current state

    mac power low-power get [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac power low-power on

Enable

    mac power low-power on [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac power low-power off

Disable

    mac power low-power off [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac power sleep-after

Set sleep timers in minutes; zero disables the timer

    mac power sleep-after [OPTIONS] <minutes>

Requirements: macOS 14+ on Apple Silicon.

## mac power awake

Keep the system awake until the timeout or Ctrl-C

    mac power awake [OPTIONS] <seconds>

Requirements: macOS 14+ on Apple Silicon.

## mac appearance get

Read the current appearance

    mac appearance get [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac appearance set

Set the appearance

    mac appearance set [OPTIONS] <mode>

Requirements: macOS 14+ on Apple Silicon.

## mac appearance wallpaper

Set wallpaper on all screens

    mac appearance wallpaper [OPTIONS] <path>

Requirements: macOS 14+ on Apple Silicon.

## mac settings list

List settings; include experimental entries with --experimental

    mac settings list [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac settings get

Read one setting

    mac settings get [OPTIONS] <key>

Requirements: macOS 14+ on Apple Silicon.

## mac settings set

Change one setting and record its previous value

    mac settings set [OPTIONS] <key> <value>

Requirements: macOS 14+ on Apple Silicon.

## mac settings undo

Undo a recorded operation; refuses conflicting newer changes

    mac settings undo [OPTIONS] <operation>

Requirements: macOS 14+ on Apple Silicon.

## mac profile save

Save selected settings to a TOML file

    mac profile save [OPTIONS] <file> <keys>...

Requirements: macOS 14+ on Apple Silicon.

## mac profile show

Read a profile

    mac profile show [OPTIONS] <file>

Requirements: macOS 14+ on Apple Silicon.

## mac profile diff

Compare a profile with current settings

    mac profile diff [OPTIONS] <file>

Requirements: macOS 14+ on Apple Silicon.

## mac profile apply

Apply a profile with rollback on failure

    mac profile apply [OPTIONS] <file>

Requirements: macOS 14+ on Apple Silicon.

## mac lock

Control the current macOS session

    mac lock [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac sleep

Control the current macOS session

    mac sleep [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac screensaver

Control the current macOS session

    mac screensaver [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac restart

Control the current macOS session

    mac restart [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac shutdown

Control the current macOS session

    mac shutdown [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac capture screenshot

Capture a screen, region, or window

    mac capture screenshot [OPTIONS]

Requirements: Screen Recording permission for screenshots.

## mac capture ocr

Recognize text in an image file

    mac capture ocr [OPTIONS] <path>

Requirements: Screen Recording permission for screenshots.

## mac clipboard read

Read clipboard text

    mac clipboard read [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac clipboard write

Write text; reads stdin if text is omitted

    mac clipboard write [OPTIONS] [text]

Requirements: macOS 14+ on Apple Silicon.

## mac clipboard clear

Clear the clipboard

    mac clipboard clear [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac system info

Show hardware and OS information

    mac system info [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac system memory

Show virtual memory statistics

    mac system memory [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac system cpu

Show CPU load averages

    mac system cpu [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac system processes

List processes by CPU use

    mac system processes [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac disk list

List disks and partitions

    mac disk list [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac disk usage

Show mounted filesystem usage

    mac disk usage [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac disk eject

Safely eject a volume or disk

    mac disk eject [OPTIONS] <target>

Requirements: macOS 14+ on Apple Silicon.

## mac backup status

Read Time Machine backup status

    mac backup status [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac backup start

Start a Time Machine backup

    mac backup start [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac files size

Calculate the size of a file or folder

    mac files size [OPTIONS] <path>

Requirements: macOS 14+ on Apple Silicon.

## mac files largest

List largest files under a folder

    mac files largest [OPTIONS] <path>

Requirements: macOS 14+ on Apple Silicon.

## mac trash size

Read Trash size

    mac trash size [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac trash empty

Empty your Trash after confirmation

    mac trash empty [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac update list

List available macOS updates

    mac update list [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac update install

Install one explicitly named update

    mac update install [OPTIONS] <label>

Requirements: macOS 14+ on Apple Silicon.

## mac weather

Open the built-in Weather app

    mac weather [OPTIONS] [COMMAND]

Requirements: macOS 14+ on Apple Silicon.

## mac weather open

Open Weather

    mac weather open [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac apps list

List installed applications and utility aliases

    mac apps list [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac apps open

Open an application by name, alias, or bundle ID

    mac apps open [OPTIONS] <name>

Requirements: macOS 14+ on Apple Silicon.

## mac calendar list

List calendars

    mac calendar list [OPTIONS]

Requirements: Calendar permission.

## mac calendar today

Read today's events in the local time zone

    mac calendar today [OPTIONS]

Requirements: Calendar permission.

## mac calendar events

Read an RFC3339 date range

    mac calendar events [OPTIONS] <start> <end>

Requirements: Calendar permission.

## mac calendar add

Create an event with RFC3339 start and end

    mac calendar add [OPTIONS] <title> <start> <end>

Requirements: Calendar permission.

## mac calendar delete

Delete one event occurrence by ID

    mac calendar delete [OPTIONS] <id>

Requirements: Calendar permission.

## mac reminders lists

List reminder lists

    mac reminders lists [OPTIONS]

Requirements: Reminders permission.

## mac reminders list

List reminders

    mac reminders list [OPTIONS]

Requirements: Reminders permission.

## mac reminders search

Search reminder titles

    mac reminders search [OPTIONS] <query>

Requirements: Reminders permission.

## mac reminders add

Create a reminder

    mac reminders add [OPTIONS] <title>

Requirements: Reminders permission.

## mac reminders complete

Complete a reminder by ID

    mac reminders complete [OPTIONS] <id>

Requirements: Reminders permission.

## mac reminders delete

Delete a reminder by ID

    mac reminders delete [OPTIONS] <id>

Requirements: Reminders permission.

## mac notes folders

List note folders

    mac notes folders [OPTIONS]

Requirements: Application Automation permission.

## mac notes list

List note titles and IDs

    mac notes list [OPTIONS]

Requirements: Application Automation permission.

## mac notes search

Search note titles

    mac notes search [OPTIONS] <query>

Requirements: Application Automation permission.

## mac notes read

Read a note by ID

    mac notes read [OPTIONS] <id>

Requirements: Application Automation permission.

## mac notes add

Create a plain-text note

    mac notes add [OPTIONS] <title> <body>

Requirements: Application Automation permission.

## mac play

Resume the system's current media session

    mac play [OPTIONS]

Requirements: An app registered with macOS Now Playing; private MediaRemote API.

## mac pause

Pause the system's current media session

    mac pause [OPTIONS]

Requirements: An app registered with macOS Now Playing; private MediaRemote API.

## mac music status

Read or control Music playback

    mac music status [OPTIONS]

Requirements: Application Automation permission.

## mac music play

Read or control Music playback

    mac music play [OPTIONS]

Requirements: Application Automation permission.

## mac music pause

Read or control Music playback

    mac music pause [OPTIONS]

Requirements: Application Automation permission.

## mac music toggle

Read or control Music playback

    mac music toggle [OPTIONS]

Requirements: Application Automation permission.

## mac music next

Read or control Music playback

    mac music next [OPTIONS]

Requirements: Application Automation permission.

## mac music previous

Read or control Music playback

    mac music previous [OPTIONS]

Requirements: Application Automation permission.

## mac shortcuts list

List available shortcuts

    mac shortcuts list [OPTIONS]

Requirements: macOS 14+ on Apple Silicon.

## mac shortcuts run

Run a named shortcut

    mac shortcuts run [OPTIONS] <name>

Requirements: macOS 14+ on Apple Silicon.
