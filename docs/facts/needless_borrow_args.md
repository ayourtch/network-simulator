# Needless Borrow for Command Arguments Fact

When passing an array of string literals to `Command::args`, the compiler/Clippy warns about unnecessary borrowing (`&["..."]`). Using a plain array (`["...", ...]`) avoids the warning and keeps the code cleaner.
