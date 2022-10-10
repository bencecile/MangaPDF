# ToDo
- Complete the implementation for formatting a paragraph
- Give a line more space if it has ruby content
- Don't put punctuation on a line by itself
    - Fix this by bringing the characters past the last break point onto the next line
        - A break point will be a space, or non-ascii non-punctuation (JA characters to start)
        - This will shorten up the previous line and should still follow the justification
    - Include JA and EN punctuation
