# Unused import from same module

`a` imports only `light` from `b`, which also exports `heavy`.
`heavy`'s return type references `Heavy` from `c`.

Module `c` reaches `Step::Load` — its file contents are read to
resolve the import target, but no keys are solved. `heavy`'s
signature is never resolved because nobody demands it: the only
Answer-level demand into `b` is `KeyExport("light")`, and `light`'s
return is annotated `int`, so the chain stops there.

## Files

`a.py`:
```python
from b import light
x = light()
```

`b.py`:
```python
from c import Heavy
def light() -> int: return 1
def heavy() -> Heavy: ...
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
a -> b::Exports(is_special_export)
a -> b::Exports(export_exists)
a -> b::Exports(get_deprecated)
a -> b::KeyExport(Name("light"))
  b -> c::Load(module_exists)
```
