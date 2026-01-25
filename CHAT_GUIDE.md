# Kayfabe Chat Interface Guide

## Welcome to Kayfabe! 🎉

Kayfabe now features a **conversational chat interface** - just like talking to Claude! No more memorizing complex command syntax.

## Getting Started

Simply run:
```bash
kayfabe
```

You'll see a chat interface with Kayfabe ready to help you!

## How It Works

Kayfabe guides you through everything conversationally. Just tell it what you want to do!

### Example: Adding a Task

**Old way (still works):**
```
/add implement-auth Add user authentication
```

**New way (conversational):**
```
You: /add
Kayfabe: I'll help you add a new task. What should we call it?
You: implement-auth
Kayfabe: Got it! Add a description for this task? (or just press Enter to skip)
You: Add user authentication
Kayfabe: ✓ Task 'implement-auth' created successfully!
```

See? Kayfabe asks for what it needs step-by-step!

### Example: Executing a Task

```
You: /execute
Kayfabe: Here are your tasks:
1. ○ implement-auth
2. ○ fix-bug-123
3. ○ write-tests

Which one would you like to execute?
You: 1
Kayfabe: Executing task 'implement-auth'...
```

You can type the task name OR just the number!

## Available Commands

### Task Management
- `/add` - Kayfabe will ask for the task name and description
- `/remove` or `/delete` - Kayfabe shows your tasks and asks which to remove
- `/execute` or `/run` - Pick a task to run
- `/list` or `/show tasks` - See all your tasks

### Project Management
- `/init` - Initialize a new project (asks for name and git URL)
- `/status` - Show project statistics

### Running Tasks
- `/execute` - Run a task in tmux
- `/attach` - Attach to a running task
- `/kill` or `/stop` - Stop a running task

### Utilities
- `/help` - Show available commands
- `/quit` or `/exit` - Leave Kayfabe

## Natural Language

You can also just talk naturally!

```
You: show me my tasks
Kayfabe: You have 3 task(s):
1. implement-auth [Pending]
2. fix-bug-123 [Active]
3. write-tests [Completed]
```

```
You: help
Kayfabe: I can help you manage tasks! Try:
• '/add' to create a new task
• '/execute' to run a task
• '/list' to see all tasks

What would you like to do?
```

## The Interface

```
┌─ Kayfabe Chat │ Ask me anything! ────────┐  ┌─ Tasks (3) ──────┐
│                                           │  │                  │
│ [12:30] Kayfabe: Welcome! How can I help?│  │ ○ implement-auth │
│ [12:31] You: show tasks                  │  │ ▶ fix-bug-123    │
│ [12:31] Kayfabe: You have 3 tasks...     │  │ ✓ write-tests    │
│                                           │  │                  │
│                                           │  │                  │
└───────────────────────────────────────────┘  └──────────────────┘
┌─ Type your message... ────────────────────────────────────────────┐
│ _                                                                  │
└────────────────────────────────────────────────────────────────────┘
```

### Layout Features

- **Left Side**: Chat conversation with Kayfabe
- **Right Side**: Quick view of your tasks (can be toggled)
- **Bottom**: Input box - just start typing!

### Keyboard Shortcuts

- `Tab` - Toggle task list visibility
- `↑/↓` - Navigate tasks in the sidebar
- `Esc` - Quit (or cancel pending command)
- `Ctrl+C` - Quick quit

## Tips & Tricks

### 1. Numbers Work Everywhere
When Kayfabe shows a numbered list, you can respond with the number:
```
You: /remove
Kayfabe: Which task would you like to remove?
1. ○ implement-auth
2. ○ fix-bug-123
You: 2
Kayfabe: Are you sure you want to remove 'fix-bug-123'? Type 'yes' to confirm.
You: yes
Kayfabe: ✓ Task 'fix-bug-123' removed successfully!
```

### 2. Cancel Anytime
If you start a command but change your mind:
```
You: /add
Kayfabe: What should we call it?
You: cancel
Kayfabe: Command cancelled. What else can I help you with?
```

Or just type `.` to cancel.

### 3. Skip Optional Fields
When Kayfabe asks for optional information, just press Enter to skip:
```
Kayfabe: Add a description? (or just press Enter to skip)
You: [Press Enter]
Kayfabe: ✓ Task created!
```

### 4. Use Shortcuts
- `/add` instead of `/add <name> <description>`
- `/execute` instead of `/execute <task-name>`
- Let Kayfabe guide you!

## Full Conversation Flow Examples

### Creating Your First Project
```
$ kayfabe
Kayfabe: Welcome to Kayfabe! I'm here to help you manage your tasks. What would you like to do?
You: /init
Kayfabe: Let's initialize a new project! What would you like to name it?
You: my-awesome-app
Kayfabe: Great! What's the Git repository URL?
You: https://github.com/myuser/awesome-app.git
Kayfabe: Initializing project 'my-awesome-app' from https://github.com/myuser/awesome-app.git...
```

### Managing Tasks
```
You: /add
Kayfabe: I'll help you add a new task. What should we call it?
You: implement-login
Kayfabe: Got it! Add a description for this task?
You: Create login page with email/password
Kayfabe: ✓ Task 'implement-login' created successfully!

You: show my tasks
Kayfabe: You have 1 task(s):
1. implement-login [Pending]
   Create login page with email/password

You: /execute
Kayfabe: Here are your tasks:
1. ○ implement-login

Which one would you like to execute?
You: implement-login
Kayfabe: Executing task 'implement-login'...
This will launch in a tmux session.
```

## The Old CLI Still Works!

Don't worry - all the original commands still work:

```bash
kayfabe add task-name -d "Description"
kayfabe list
kayfabe execute task-name
kayfabe remove task-name
```

But the chat interface is **way more fun** and easier to use!

## Why Chat Interface?

- **No memorization** - Kayfabe asks for what it needs
- **Forgiving** - Typos? Natural language? No problem!
- **Interactive** - See your tasks while you chat
- **Guided** - Never wonder what to do next
- **Friendly** - Feels like talking to an AI assistant (because it is!)

## Start Chatting!

```bash
kayfabe
```

Type `/help` if you get stuck, or just ask Kayfabe what it can do!

Happy task managing! 🚀
