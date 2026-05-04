# Bare import only demands Load on transitive dep

`a` imports `light` from `b`. `b` has `import c` (bare import, not
`from c import ...`). `a` only uses `light()` which doesn't involve `c`.

`module_exists(c)` now demands `Step::Load` on `c` instead of
`Step::Exports`, so `c` can stay at `Load` when nothing else forces
it higher. This is the expected behaviour — `b` never used `c` in a
way that `a` needed, and the demand tree confirms no other edge
reaches `c`.

## Files

`a.py`:
```python
from b import light
x = light()
```

`b.py`:
```python
import c
def light() -> int: return 1
```

`c.py`:
```python
class Heavy:
    x: int = 1
```

## Check `a.py`

```expected
a: Solutions
b: Answers
c: Load

(160 builtin demands hidden)
a -> b::Load(module_exists)
a -> b::Exports(export_exists)
a -> b::Exports(is_special_export)
a -> b::Exports(get_deprecated)
a -> b::KeyExport(Name("light"))
  b -> c::Load(module_exists)
```
