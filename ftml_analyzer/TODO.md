# TODO

- [x] Fix recursive `Block` highlights mismatch in `InlineMarks`

```ftml
**Bold [[footnote]] something footnote [[/footnote]] **
```

- [ ] Add highlights for table

```ftml
||~ table head ||~ table head2 ||
|| table row   || table row2   ||
```

- [ ] Add highlights for url token types

```ftml
[https://example.com example]
```

- [ ] Expand spans

```ftml
**bold //italic// bold**
```

- [ ] Implement a lint system for formatting the code 
- [ ] Implement a test harness to make tests more readable

## Known issues

`InlineBlocks` have incorrect highlights while parsing code like:
```ftml
[[include :path:to:page
|var = value
]]
```
If there's an empty space between `=` and value, the value type would become `Attr` rather than `Value`.
Generally, this is acceptable most of time, will implement an lint/fmt for that.

