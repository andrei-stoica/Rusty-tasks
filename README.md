# Rusty-tasks
A rewrite of an older CLI todo system without python and regex

## Goals

- [ ] BLAZING FAST!
- [ ] replace python with rust
- [ ] replace regex with a markdown parsing library
- [ ] learn some rust along the way

## Ideas

If you've used the bullet journal technique before, think if this as the
digital, automated version of that.

- Accessible from anywhere in the terminal (TMUX, bare shell, etc)
    * works really well with a drop-down terminal
- Takes advantage of default editor (vim, emacs, etc)
- Take advantage of Markdown for formatting
- Organize tasks at different levels of urgency
    * Daily - Must get done now
    * Weekly - Must get done in the near future
    * Monthly - I'm going to forget if I don't write it down but It's a ways away
- Every day start a new file 
    * files should be dated
- Carry over uncompleted tasks from previous day
