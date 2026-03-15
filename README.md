# Taskmaster
Taskmaster is a simple terminal interface for tracking your tasks. I built it as a hobby project to explore rust, and some popular crates including [ratatui](https://ratatui.rs/), [tokio](https://tokio.rs/), [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/) and [clap](https://docs.rs/clap/latest/clap/). Your todos are preserved in `SQLite` databases following the [XDG Specification](https://specifications.freedesktop.org/basedir/latest/). Furthermore, all of the todo's that you've completed are also preserved and displayed for you. 

# Usage

## TUI
The TUI interface for `taskmaster` will serve the majority of your use cases. While in the TUI, there are several modes and commands to keep in mind. Each command is performed by simply pressing the associated key. At this point, `taskmaster` does not support formal editing modes, or command leaders/prefixes. 

### Display mode 
Display mode is the primary viewing mode for taskmaster. This is where you will be able to visualize your list of todo items, as well as your completed todos. There are several commands to keep in mind for interacting with your lists. Note that not all commands work for both lists, and some will only work on your active todo list.  

| Key | Applicable List | Description | 
| ------------- | ------------- | -------------- | 
| `Esc` \| `q\|Q` \| `Ctl + C`  | All | Exit application |
| `a\|A` |All| Enter add mode |
| `r` |All| Remove selected todo |
| `c` |todo list| Complete the selected todo and move to completed list |
| `e` |todo list| Edit the selected todo |

### Add mode 
Add mode is where you will be adding and editing your tasks. The fields are all optional, however it's recommended you at least include a name. Everything else is just information for yourself, so feel free to include or ignore. The are relatively few add mode commands other than closing add mode to return to display mode, and submitting the new todo. 

| Key | Description | 
| ------------- | -------------- | 
| `Esc` | Return to display mode |
| `Enter` | Submit new todo and return to display mode |



## CLI
While not the primary mode, the CLI offers the same options as the TUI interface. You can use the following commands if you want to interact with your todo list without opening the application. Note that both completing and removing a todo require you to specify an `id` which can be found using the `taskmaster list` command.  

To add a todo:

```bash
taskmaster add -n <name> -r <report_to> -d <due_date>
```

To list your todos:
```bash
taskmaster list
```

To complete a todo:
```bash
taskmaster complete --id <id> 
```

To remove a todo without completing:
```bash
taskmaster remove -id <id>
```

# FAQ
## Why `taskmaster`? 
This application actually started under the name `todui`, which I thought was incredibly clever, only to discover another project existed under this name. My fiancé's and I love the show [Taskmaster](https://www.taskmaster.tv/), and it seemed like a fortunate fit for this project. 

# License
taskmaster is licensed under the Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International license. 


