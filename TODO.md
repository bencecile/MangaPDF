# ToDo
- Complete the implementation for formatting a paragraph
- Escape 0x28 `(`, 0x29 `)`, 0x5C `\` with 0x5C `\` in the PDF text
- Automatically convert non-ascii characters to UTF-16 when showing them in a PDF
    - Should probably use a separate Tj operator for all consecutive UTF-16 characters
- Give a line more space if it has ruby content
