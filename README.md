# `file_obliterator`

A context menu and CLI tool for Windows that unlocks/deletes files and folders that are taken up by other processes. No slow UI allowed.

If you've ever felt like you just want the files obliterated, they're gone now.

I made this because all the other tools come with UI that takes forever to launch and elevate. The obliterate button wouldn't have a confirmation if it wasn't so dangerous.

## Usage

The setup will cleanly restart `explorer.exe`, so your open folders won't be lost.

```bash
cargo install --git https://github.com/Kyza/file_obliterator
# Runs the setup.
file_obliterator
# Or more verbose:
# file_obliterator -a setup
```

You now have the obliterate and unlock buttons in the file explorer context menu.

Because it's compiled to run as administrator, it won't display any output unless you run it inside an elevated terminal.

```bash
file_obliterator -h
```

If you uninstall the tool, don't forget to run the unsetup to remove the context menu items.

```bash
file_obliterator -a unsetup
```

## Contributing

PRs are welcome, especially for other operating systems.
