# TODO

- [x] Fix recursive `Block` highlights mismatch in `InlineMarks`

```ftml
**Bold [[footnote]] something footnote [[/footnote]] **
```

- [x] Add highlights for table

```ftml
||~ table head ||~ table head2 ||
|| table row   || table row2   ||
```

- [x] Add highlights for url token types

```ftml
[https://example.com example]
```

- [x] Expand spans

```ftml
**bold //italic// bold**
```

- [x] Add auto-completion service for "wikitext"
- [ ] Add autofix service for "wikitext"

## Pending

- [ ] Implement a lint system for formatting the code
- [ ] Implement a test harness to make tests more readable

## Known issues

### InlineBlocks

`InlineBlocks` have incorrect highlights while parsing code like:

```ftml
[[include :path:to:page
|var = value
]]
```

If there's an empty space between `=` and value, the value type would become `Attr` rather than `Value`.
Generally, this is acceptable most time, will implement a lint/formatter for that.

### Note

> [!NOTE]
> Mostly finished auto-completion, I don't known why ftml returns an error for `[[note]][[/note]]` blocks
> I double checked my implementation and tests in ftml, but everything goes fine... This is documented here as a known issue.

