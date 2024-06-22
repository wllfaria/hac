Output extra/new-request.gif
Hide
    Set Shell zsh

    Set Theme { "name": "Kanagawa", "cursor": "#DCD7BA", "selection": "#2A2A37", "background": "#181616", "foreground": "#DCD7BA", "black": "#1F1F28", "blue": "#658594", "cyan": "#9CABCA", "green": "#76946A", "magenta": "#957FB8", "red": "#E82424", "white": "#DCD7BA", "yellow": "#FF9E3B", "brightBlack": "#2A2A37", "brightBlue": "#7FB4CA", "brightCyan": "#A3D4D5", "brightGreen": "#98BB6C", "brightMagenta": "#D27E99", "brightRed": "#FF5D62", "brightWhite": "#DCD7BA", "brightYellow": "#E6C384" }

    Set FontSize 12
    Set Width 1200
    Set Height 600

    Set Padding 0
    Set FontFamily "JetBrains Mono"

    Set WindowBar Colorful
    Set WindowBarSize 50

    Type "cargo run -q -- --dry-run"
    Enter
    # enter the editor
    Sleep 3s
    Enter
    Sleep 3s
Show
Sleep 2s
# select sidebar
Enter
Sleep 500ms
# enter dir form
Type "d"
Sleep 1s
Type@100ms "users"
Sleep 1s
# create the form
Enter
Sleep 1s
# enter request form
Type "n"
Sleep 1s
Type@100ms "register"
Sleep 1s
Tab
Sleep 500ms
# should be a post
Type "2"
Sleep 500ms
Tab
# enter parent selector
Space
Sleep 2s
Enter
Sleep 2s
# create the request
Enter
Sleep 2s
Type "j"
Sleep 300ms
Enter
Sleep 300ms
Type "j"
Sleep 2s
Enter
Sleep 1s
Escape
Sleep 300ms
Tab
Sleep 300ms
Tab
Enter
Sleep 1s
Tab
Sleep 1s
Type "n"
Sleep 1s
Type "Authorization"
Sleep 1s
Tab
Sleep 200ms
Type "Bearer your token"
Sleep 3s
