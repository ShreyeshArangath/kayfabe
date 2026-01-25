# Interactive Mode

## Overview

Kayfabe now supports an interactive mode for CLI commands, making it much easier to select tasks without needing to type or copy exact task names. This is particularly useful when:
- Task names are too long to see or copy easily
- You want to quickly browse available tasks
- You're unsure of the exact task name

## Usage

All task-related commands now support the `--interactive` or `-i` flag:

### Execute Command
```bash
# Traditional way (requires exact task name)
kayfabe execute my-long-task-name-that-is-hard-to-type

# Interactive way
kayfabe execute -i
# or
kayfabe execute --interactive
```

When using interactive mode, you'll see a filterable list of pending tasks that you can:
- Navigate with arrow keys (↑↓)
- Filter by typing
- Select with Enter

### Remove Command
```bash
# Interactive removal with confirmation
kayfabe remove -i

# Interactive removal without confirmation
kayfabe remove -i --force
```

Shows all tasks with full details, making it easy to identify which one to remove.

### Attach Command
```bash
# Attach to a running task interactively
kayfabe attach -i
```

Filters to show only active tasks, so you can quickly attach to running sessions.

### Kill Command
```bash
# Kill a running task interactively
kayfabe kill -i
```

Shows only active tasks that can be killed.

### Logs Command
```bash
# View logs interactively
kayfabe logs -i

# View and follow logs interactively
kayfabe logs -i --follow
```

Shows all tasks, allowing you to select which one's logs to view.

## Interactive Selection Features

The interactive selector provides:

1. **Full Task Information**: Each task is displayed with:
   - Status icon (○ Pending, ▶ Active, ✓ Completed, ⊗ Archived)
   - Task name (in full, no truncation)
   - Status label
   - Description (if available, truncated to 50 chars)

2. **Keyboard Navigation**:
   - `↑` / `↓`: Move up/down through tasks
   - `Enter`: Select the highlighted task
   - Type to filter: Start typing to search/filter tasks by name or description
   - `Esc`: Cancel selection

3. **Smart Filtering**: By default, commands filter to relevant tasks:
   - `execute -i`: Shows only pending tasks
   - `attach -i`: Shows only active tasks
   - `kill -i`: Shows only active tasks
   - `logs -i`: Shows all tasks
   - `remove -i`: Shows all tasks

4. **Confirmation Dialogs**: When needed (like for removal), you'll get a proper confirmation prompt instead of having to type 'y' or 'n'.

## Examples

### Typical Interactive Workflow

```bash
# 1. Add a task (normal CLI)
$ kayfabe add "implement-user-authentication" -d "Add JWT-based auth"

# 2. Execute it interactively
$ kayfabe execute -i
? Select task to execute: (Use arrow keys)
> ○ implement-user-authentication [Pending] - Add JWT-based auth
  ○ fix-database-migration [Pending] - Fix schema version
  ○ update-documentation [Pending]

# 3. Check logs interactively
$ kayfabe logs -i
? Select task to view logs: (Use arrow keys)
  ○ implement-user-authentication [Pending] - Add JWT-based auth
> ▶ fix-database-migration [Active] - Fix schema version
  ✓ update-documentation [Completed]

# 4. Remove completed task interactively
$ kayfabe remove -i
? Select task to remove: (Use arrow keys)
  ○ implement-user-authentication [Pending] - Add JWT-based auth
  ▶ fix-database-migration [Active] - Fix schema version
> ✓ update-documentation [Completed]

? Are you sure you want to remove task 'update-documentation'?
Worktree: /path/to/worktrees/update-documentation
This action cannot be undone. (y/N) y

✓ Task 'update-documentation' removed successfully
```

## Benefits

1. **No More Copy/Paste Issues**: See the full task name and select it with arrow keys
2. **Better UX**: Visual feedback and easy navigation
3. **Faster Workflow**: Quickly browse and select tasks without typing
4. **Error Prevention**: See exactly what you're selecting before confirming
5. **Backward Compatible**: Original CLI interface still works - just don't use `-i`

## Backward Compatibility

The traditional CLI interface is still fully supported. You can continue using:
```bash
kayfabe execute my-task
kayfabe remove my-task --force
```

The interactive mode is completely optional and additive to the existing functionality.
