pub static DEFAULT_CONFIG: &str = r##"
[editor_keys.normal]
"u" = "Undo"
"n" = "FindNext"
"S-N" = "FindPrevious"
"w" = "NextWord"
"b" = "PreviousWord"
"h" = "MoveLeft"
"Left" = "MoveLeft"
"j" = "MoveDown"
"Down" = "MoveDown"
"k" = "MoveUp"
"Up" = "MoveUp"
"l" = "MoveRight"
"Right" = "MoveRight"
"S-G" = "MoveToBottom"
"g" = { "g" = "MoveToTop" }
"$" = "MoveToLineEnd"
"End" = "MoveToLineEnd"
"Home" = "MoveToLineStart"
"0" = "MoveToLineStart"
"C-d" = "PageDown"
"C-u" = "PageUp"
"S-D" = "DeleteUntilEOL"
"x" = "DeleteCurrentChar"
"o" = ["InsertLineBelow", "InsertAtEOL"]
"S-O" = "InsertLineAbove"
"p" = "PasteBelow"
"a" = "InsertAhead"
"i" = { EnterMode = "Insert" }
"S-I" = ["MoveToLineStart", { EnterMode = "Insert" }]
"S-A" = "InsertAtEOL"
"S-B" = "MoveAfterWhitespaceReverse"
"S-W" = "MoveAfterWhitespace"
"S-X" = "DeletePreviousNonWrapping"
"%" = "JumpToClosing"
"{" = "JumpToEmptyLineAbove"
"}" = "JumpToEmptyLineBelow"

[editor_keys.normal.d]
"w" = "DeleteWord"
"d" = "DeleteLine"
"b" = "DeleteBack"
"j" = "DeleteCurrAndBelow"
"k" = "DeleteCurrAndAbove"
"l" = "DeleteCurrentChar"
"h" = "DeletePreviousChar"

[editor_keys.insert]
"Tab" = "InsertTab"
"Enter" = "InsertLine"
"Backspace" = "DeletePreviousChar"
"Esc" = { EnterMode = "Normal" }
"C-c" = { EnterMode = "Normal" }
"C-W" = "DeleteBack"
"##;
