# Rusty-tasks

A personal task management system built on plain text and works in your favorite editor


## The General Idea

If you've used the bullet journal technique before, think of this as the
digital, automated version of that.

- Accessible from anywhere in the terminal (TMUX, bare shell, etc.)
    * works really well with a drop-down terminal
- Take advantage of your favorite editor (vim, Emacs, etc.)
- Take advantage of plain text formats (Markdown)
- Organize tasks at different levels of urgency
    * Daily - Must get done now
    * Weekly - Must get done in the near future
    * Monthly - I'm going to forget if I don't write it down, but It's a ways away
- Every day start a new file 
    * files should be dated
- Carry over uncompleted tasks from the previous day


## Installing 

```bash
git clone https://github.com/andrei-stoica/rusty-tasks.git
cd rusty-task
cargo install --path .
```

Alternatively, there is a binary download for AMD64 Linux machines available 
on the [releases page](https://github.com/andrei-stoica/rusty-tasks/releases).
Just drop that anywhere on you PATH. I recommend adding `~/bin` to your PATH
and dropping the executable there.

If you are not on a AMD64 Linux machine, you will need to build from source.
I have not tested this on other platforms, so I hesitate to provide binaries
for them.

## Usage
***WARNING:*** *This documentation can be ahead of the releases on the GH release page*
```help
Usage: rusty-tasks [OPTIONS]

Options:
  -c, --config <FILE>        set config file to use
  -C, --current-config       show current config file
  -p, --previous <PREVIOUS>  view previous day's notes [default: 0]
  -l, --list                 list closest files to date
  -n, --number <NUMBER>      number of files to list [default: 5]
  -L, --list-all             list closest files to date
  -v, --verbose...           increase logging level
  -h, --help                 Print help
  -V, --version              Print version
```

Just use `rust-task` to access today's notes file.

Use `rust-task -p <n>` to access a previous day's file where `<n>` is the number
of days back you want to go. If a file does not exist for that day, it will
default to the closest to that date. A value of 0 represents today's file.

Specify a custom config location with `-c`, otherwise, it will scan for a config
in the locations specified in the [config section](#config). If no config
exists it will create one. To see what config is being loaded you can use `-C`.

To list your existing notes you can use `-L`. For a subset of these use
`-l` combined with `-n` to specify the number of files to list. This will be
the closest `n` files to the specified date, which is today by default. Specify
the target date using the `-p` as mentioned earlier

## Config

The config should be located in the following locations:

- `~/.config/rusty_task.json`
- `~/.rusty_task.json`
- `$PWD/.rusty_task.json`

If there is no config it will be created at `~/.config/rusty_task.json`.

Example config:
```
{
  "editor": "nano",
  "sections": [
    "Daily",
    "Weekly",
    "Monthly"
  ],
  "notes_dir": "~/notes"
}
```

- `editor` is the executable that will be launched to modify the notes file
- `sections` is a list of Sections that will be carried over from the previous
day's notes
    * only uncompleted tasks are carried over
    * You can use other sections for scratch space and other journaling tasks
- `notes_dir` is the directory that stores your daily notes 
    * this could be set to your obsidian vault if you want it to work with
      all of your other notes (I recommend checking out [obsidian.nvim](https://github.com/epwalsh/obsidian.nvim)
      if you want to interact with an obsidian vault in neovim)

