# ftml_handler

This crate using ftml's tokenizer and parse error to provide highlights.

### Known issues

There's might some false positives warnings on `include` blocks.

```ftml
[[include :scp-wiki-cn:theme:suprematism evernight=true |no-banner=true]]
[!--                                     ^^^^^^^^^^^^ NoSuchBlock     
Because I didn't implement the includer for code highlighter, so it won't works on "includes".
--]
```
