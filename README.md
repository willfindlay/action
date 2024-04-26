# Action

Action is an APM counter designed to be used as a browser source in OBS.

## How It Works

Action uses WinAPI hooks to count the number of mouse button presses and keystrokes you made each second. It measures over a sliding window of length 60 (for a full minute in total) and uses this to compute the average. A simple web server then serves the APM counter over `localhost:3333`, which you can then add as a browser source to OBS.

## Installation 

You have two options for installation:

1. Build from source and optionally add a shortcut to your startup folder. This is recommended for technically proficient users. Simply clone and run `cargo build --release`. You may then optionally create a shortcut to the resulting executable in your startup folder.
2. Use the [provided installer][installer] in the latest release.

**IMPORTANT WARNING:** See the section below about adding a Windows Defender exclusion **BEFORE** you attempt to install. Otherwise, the executable will be automatically removed.

## Windows Defender Exclusion

Because of the sensitive nature of this application (counting keystrokes and mouse clicks), you will need to add a Windows Defender Exclusion before installing. You can do so as follows:

1. Go to Start > Settings > Privacy & Security > Windows Security > Virus & threat protection > Virus & threat protection settings > Manage settings > Add or remove exclusions
2. Click the `+` button and select "Folder"
3. Enter "%LOCALAPPDATA%\Programs\Action" in the top bar, then click "Select Folder"

If you are building from source, replace the path in step 3 with your `target` directory.

## Configuration

The default port is `3333`, but this can be configured using the environment variable `ACTION_PORT`.

To add your APM to OBS, add a browser source and then simply set the URL to `localhost:3333`. You can add some custom CSS to style the APM counter by selecting the apm class. For example, you could do something like the following to make it orange with an outline:

```css
.apm {color: #bf6330; -webkit-text-stroke: 4px #1f2525;}
```

[installer]: https://github.com/willfindlay/action/releases/download/v0.1.0/ActionSetup.exe