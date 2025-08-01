/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::test::util::TestEnv;
use crate::testcase;

testcase!(
    test_if_simple,
    r#"
from typing import assert_type, Literal
def b() -> bool:
    return True
if b():
    x = 100
else:
    x = "test"
y = x
assert_type(y, Literal['test', 100])
"#,
);

testcase!(
    test_if_else,
    r#"
from typing import assert_type, Literal
def b() -> bool:
    return True
if b():
    x = 100
elif b():
    x = "test"
else:
    x = True
y = x
assert_type(y, Literal['test', 100, True])
"#,
);

testcase!(
    test_if_only,
    r#"
from typing import assert_type, Literal
def b() -> bool:
    return True
x = 7
if b():
    x = 100
y = x
assert_type(y, Literal[7, 100])
"#,
);

testcase!(
    test_while_simple,
    r#"
from typing import assert_type, Literal
def f(condition) -> None:
    x = None
    while condition():
        assert_type(x, Literal["hello world"] | None)
        x = "hello world"
        assert_type(x, Literal["hello world"])
    assert_type(x, Literal["hello world"] | None)
    "#,
);

testcase!(
    bug = "A recursive redefinition in a loop does not work as before after first-use variable pinning.",
    test_while_infinite,
    r#"
from typing import assert_type, Any, Literal
def f(condition) -> None:
    x = 1
    while condition():  # E: `Literal[1] | list[int]` is not assignable to `int`
        assert_type(x, Literal[1] | list[int])
        x = [x]
        assert_type(x, list[int])
    assert_type(x, Literal[1] | list[int])
    "#,
);

testcase!(
    test_while_noop,
    r#"
from typing import assert_type, Literal
def f(condition) -> None:
    x = 1
    while condition():
        pass
    assert_type(x, Literal[1])
    "#,
);

testcase!(
    test_while_fancy_noop,
    r#"
from typing import assert_type, Any, Literal
def f(condition) -> None:
    x = 1
    while condition():
        x = x
    assert_type(x, Literal[1])
    "#,
);

testcase!(
    test_while_if,
    r#"
from typing import assert_type, Any, Literal
def f(condition1, condition2) -> None:
    x = None
    while condition1():
        if condition2():
            x = "hello"
    assert_type(x, Literal['hello'] | None)
    "#,
);

testcase!(
    test_while_two_vars,
    r#"
from typing import assert_type, Any, Literal
def f(cond1, cond2, cond3) -> None:
    x = 1
    y = ""
    while cond1():
        if cond2():
            x = y
        if cond3():
            y = x
    assert_type(x, Literal["", 1])
    assert_type(y, Literal["", 1])
    "#,
);

testcase!(
    test_while_else,
    r#"
from typing import assert_type, Literal
def f(condition) -> None:
    x = None
    while condition():
        x = 1
    else:
        x = ""
    assert_type(x, Literal[""])
    "#,
);

testcase!(
    test_while_break_else,
    r#"
from typing import assert_type, Any, Literal
def f(cond1, cond2) -> None:
    x = None
    while cond1():
        if cond2():
            x = "value"
            break
        else:
            x = "overwritten"
    else:
        assert_type(x, Literal["overwritten"] | None)
        x = "default"
    assert_type(x, Literal["default", "value"])
    "#,
);

testcase!(
    test_while_else_while,
    r#"
while False:
    x = 0
else:
    while False:
        x = 1
    "#,
);

testcase!(
    test_while_reassignment_with_annotation,
    r#"
from typing import assert_type, Literal
def f(cond):
    x: int = 0
    while cond():
        x: int = 1
    assert_type(x, int)
    "#,
);

testcase!(
    test_for_simple,
    r#"
from typing import assert_type
def f(x: list[int]) -> None:
    for i in x:
        assert_type(i, int)
    assert_type(i, int)
    "#,
);

testcase!(
    test_for_tuple,
    r#"
from typing import assert_type
def f(x: tuple[int, str]) -> None:
    for i in x:
        assert_type(i, int | str)
    "#,
);

testcase!(
    test_for_literal_string,
    r#"
from typing import assert_type, LiteralString
for i in "abcd":
    assert_type(i, LiteralString)
    "#,
);

testcase!(
    test_for_any,
    r#"
from typing import Any, assert_type
def f(x: Any):
    for i in x:
        assert_type(i, Any)
    "#,
);

testcase!(
    test_for_reassign,
    r#"
from typing import assert_type
def f(x: list[int]):
    y = None
    for i in x:
        y = i
    assert_type(y, int | None)
    "#,
);

testcase!(
    test_for_else_reassign,
    r#"
from typing import assert_type, Literal
def f(x: list[int]):
    y = None
    for i in x:
        y = i
    else:
        y = 'done'
    assert_type(y, Literal['done'])
    "#,
);

testcase!(
    test_for_multiple_targets,
    r#"
from typing import assert_type
def f(x: list[tuple[int, str]]) -> None:
    for (i, j) in x:
        assert_type(i, int)
        assert_type(j, str)
    "#,
);

testcase!(
    test_for_scope,
    r#"
from typing import assert_type
def f(x: list[int]) -> None:
    for i in x:
        pass
    assert_type(i, int)
    "#,
);

testcase!(
    test_for_target_annot_compatible,
    r#"
def f(x: list[int]) -> None:
    i: int = 0
    for i in x:
        pass
    "#,
);

testcase!(
    test_for_target_annot_incompatible,
    r#"
def f(x: list[int]) -> None:
    i: str = ""
    for i in x: # E: Cannot use variable `i` with type `str` to iterate through `list[int]`
        pass
    "#,
);

testcase!(
    test_listcomp_simple,
    r#"
from typing import assert_type
y = [x for x in [1, 2, 3]]
assert_type(y, list[int])
    "#,
);

testcase!(
    test_listcomp_no_leak,
    r#"
def f():
    y = [x for x in [1, 2, 3]]
    return x  # E: Could not find name `x`
    "#,
);

testcase!(
    test_listcomp_no_overwrite,
    r#"
from typing import assert_type
x = None
y = [x for x in [1, 2, 3]]
assert_type(x, None)
    "#,
);

testcase!(
    test_listcomp_read_from_outer_scope,
    r#"
from typing import assert_type
x = None
y = [x for _ in [1, 2, 3]]
assert_type(y, list[None])
    "#,
);

testcase!(
    test_listcomp_iter_error,
    r#"
class C:
    pass
[None for x in C.error]  # E: Class `C` has no class attribute `error`
    "#,
);

testcase!(
    test_listcomp_if_error,
    r#"
class C:
    pass
def f(x):
    [None for y in x if "5" + 5]  # E: `+` is not supported between `Literal['5']` and `Literal[5]`
    "#,
);

testcase!(
    test_listcomp_target_error,
    r#"
def f(x: list[tuple[int]]):
    [None for (y, z) in x]  # E: Cannot unpack
    "#,
);

testcase!(
    test_listcomp_splat,
    r#"
from typing import assert_type
def f(x: list[tuple[int, str, bool]]):
    z = [y for (_, *y) in x]
    assert_type(z, list[list[bool | str]])
    "#,
);

testcase!(
    test_setcomp,
    r#"
from typing import assert_type
y = {x for x in [1, 2, 3]}
assert_type(y, set[int])
    "#,
);

testcase!(
    test_dictcomp,
    r#"
from typing import assert_type
def f(x: list[tuple[str, int]]):
    d = {y: z for (y, z) in x}
    assert_type(d, dict[str, int])
    "#,
);

testcase!(
    test_generator,
    r#"
from typing import assert_type, Generator
y = (x for x in [1, 2, 3])
assert_type(y, Generator[int, None, None])
    "#,
);

testcase!(
    test_bad_loop_command,
    r#"
break  # E: Cannot `break` outside loop
continue  # E: Cannot `continue` outside loop
    "#,
);

testcase!(
    test_break,
    r#"
from typing import assert_type, Literal
def f(cond):
    x = None
    for i in [1, 2, 3]:
        x = i
        if cond():
            break
        x = "hello world"
    assert_type(x, Literal["hello world"] | int | None)
    "#,
);

testcase!(
    test_continue,
    r#"
from typing import assert_type, Literal
def f(cond1, cond2):
    x = None
    while cond1():
        x = 1
        if cond2():
            x = 2
            continue
        assert_type(x, Literal[1])
        x = "hello world"
    assert_type(x, Literal["hello world", 2] | None)
    "#,
);

testcase!(
    test_early_return,
    r#"
from typing import assert_type, Literal
def f(x):
    if x:
        y = 1
        return
    else:
        y = "2"
    assert_type(y, Literal["2"])
    "#,
);

testcase!(
    test_return_in_for,
    r#"
def f(x: str):
    for c in x:
        return
    "#,
);

testcase!(
    test_flow_scope_type,
    r#"
from typing import assert_type

# C itself is in scope, which means it ends up bound to a Phi
# which can cause confusion as both a type and a value
class C: pass

c = C()

while True:
    if True:
        c = C()

assert_type(c, C)
    "#,
);

testcase!(
    test_flow_crash,
    r#"
def test():
    while False:
        if False:
            x: int
        else:
            x: int
            if False:
                continue
"#,
);

testcase!(
    test_flow_crash2,
    r#"
def magic_breakage(argument):
    for it in []:
        continue
        break
    else:
        raise
"#,
);

testcase!(
    test_try,
    r#"
from typing import assert_type, Literal

try:
    x = 1
except:
    x = 2

assert_type(x, Literal[1, 2])
"#,
);

testcase!(
    test_exception_handler,
    r#"
from typing import reveal_type

class Exception1(Exception): pass
class Exception2(Exception): pass

x1: tuple[type[Exception], ...] = (Exception1, Exception2)
x2 = (Exception1, Exception2)

try:
    pass
except int as e1:  # E: Invalid exception class: `int` does not inherit from `BaseException`
    reveal_type(e1)  # E: revealed type: int
except int:  # E: Invalid exception class
    pass
except Exception as e2:
    reveal_type(e2)  # E: revealed type: Exception
except ExceptionGroup as e3:
    reveal_type(e3)  # E: revealed type: ExceptionGroup[Exception]
except (Exception1, Exception2) as e4:
    reveal_type(e4)  # E: revealed type: Exception1 | Exception2
except Exception1 as e5:
    reveal_type(e5)  # E: revealed type: Exception1
except x1 as e6:
    reveal_type(e6)  # E: revealed type: Exception
except x2 as e7:
    reveal_type(e7)  # E: revealed type: Exception1 | Exception2
"#,
);

testcase!(
    test_exception_group_handler,
    r#"
from typing import reveal_type

class Exception1(Exception): pass
class Exception2(Exception): pass

try:
    pass
except* int as e1:  # E: Invalid exception class
    reveal_type(e1)  # E: revealed type: ExceptionGroup[int]
except* Exception as e2:
    reveal_type(e2)  # E: revealed type: ExceptionGroup[Exception]
except* ExceptionGroup as e3:  # E: Exception handler annotation in `except*` clause may not extend `BaseExceptionGroup`
    reveal_type(e3)  # E: ExceptionGroup[ExceptionGroup[Exception]]
except* (Exception1, Exception2) as e4:
    reveal_type(e4)  # E: ExceptionGroup[Exception1 | Exception2]
except* Exception1 as e5:
    reveal_type(e5)  # E: ExceptionGroup[Exception1]
"#,
);

testcase!(
    test_try_else,
    r#"
from typing import assert_type, Literal

try:
    x = 1
except:
    x = 2
else:
    x = 3

assert_type(x, Literal[2, 3])
"#,
);

testcase!(
    test_try_finally,
    r#"
from typing import assert_type, Literal

try:
    x = 1
except:
    x = 2
finally:
    x = 3

assert_type(x, Literal[3])
"#,
);

testcase!(
    test_match,
    r#"
from typing import assert_type

def point() -> int:
    return 3

match point():
    case 1:
        x = 8
    case q:
        x = q
assert_type(x, int)
"#,
);

testcase!(
    test_match_narrow_simple,
    r#"
from typing import assert_type, Literal

def test(x: int):
    match x:
        case 1:
            assert_type(x, Literal[1])
        case 2 as q:
            assert_type(x, int)
            assert_type(q, Literal[2])
        case q:
            assert_type(x, int)
            assert_type(q, int)

x: object = object()
match x:
    case int():
        assert_type(x, int)

y: int | str = 1
match y:
    case str():
        assert_type(y, str)
"#,
);

testcase!(
    bug = "does not detect unreachable branches based on nested patterns",
    test_match_narrow_len,
    r#"
from typing import assert_type, Never

def foo(x: tuple[int, int] | tuple[str]):
    match x:
        case [x0]:
            assert_type(x, tuple[str])
            assert_type(x0, str)
    match x:
        case [x0, x1]:
            assert_type(x, tuple[int, int])
            assert_type(x0, int)
            assert_type(x1, int)
    match x:
        # these two cases should be impossible to match
        case [str(), str()]:
            assert_type(x, tuple[int, int])
        case [int()]:
            assert_type(x, tuple[str])
"#,
);

testcase!(
    test_match_mapping,
    r#"
from typing import assert_type

x: dict[str, int] = { "a": 1, "b": 2, "c": 3 }
match x:
    case { "a": 1, "b": y, **c }:
        assert_type(y, int)
        assert_type(c, dict[str, int])

y: dict[str, object] = {}
match y:
    case { "a": int() }:
        assert_type(y["a"], int)
"#,
);

testcase!(
    test_empty_loop,
    r#"
# These generate syntax that is illegal, but reachable with parser error recovery

for x in []:
pass  # E: Expected an indented block

while True:
pass  # E: Expected an indented block
"#,
);

testcase!(
    test_match_implicit_return,
    r#"
def test1(x: int) -> int:
    match x:
        case _:
            return 1
def test2(x: int) -> int:  # E: Function declared to return `int`, but one or more paths are missing an explicit `return`
    match x:
        case 1:
            return 1
"#,
);

testcase!(
    test_match_class_narrow,
    r#"
from typing import assert_type

class A:
    x: int
    y: str
    __match_args__ = ("x", "y")

class B:
    x: int
    y: str
    __match_args__ = ("x", "y")

class C:
    x: int
    y: str
    __match_args__ = ("x", "y")

def fun(x: A | B | C) -> None:
    match x:
        case A(1, "a"):
            assert_type(x, A)
    match x:
        case B(2, "b"):
            assert_type(x, B)
    match x:
        case B(3, "B") as y:
            assert_type(x, A | B | C)
            assert_type(y, B)
    match x:
        case A(1, "a") | B(2, "b"):
            assert_type(x, A | B)
"#,
);

testcase!(
    test_match_class,
    r#"
from typing import assert_type

class Foo:
    x: int
    y: str
    __match_args__ = ("x", "y")

class Bar:
    x: int
    y: str

class Baz:
    x: int
    y: str
    __match_args__ = (1, 2)

def fun(foo: Foo, bar: Bar, baz: Baz) -> None:
    match foo:
        case Foo(1, "a"):
            pass
        case Foo(a, b):
            assert_type(a, int)
            assert_type(b, str)
        case Foo(x = b, y = a):
            assert_type(a, str)
            assert_type(b, int)
        case Foo(a, b, c):  # E: Cannot match positional sub-patterns in `Foo`\n  Index 2 out of range for `__match_args__`
            pass
    match bar:
        case Bar(1):  # E: Object of class `Bar` has no attribute `__match_args__`
            pass
        case Bar(a):  # E: Object of class `Bar` has no attribute `__match_args__`
            pass
        case Bar(x = a):
            assert_type(a, int)
    match baz:
        case Baz(1):  # E: Expected literal string in `__match_args__`
            pass
"#,
);

testcase!(
    test_match_sequence_len,
    r#"
from typing import assert_type
def test(x: tuple[object] | tuple[object, object] | list[object]) -> None:
    match x:
        case [int()]:
            assert_type(x[0], int)
        case [a]:
            assert_type(x, tuple[object] | list[object])
        case [a, b]:
            assert_type(x, tuple[object, object] | list[object])
"#,
);

testcase!(
    test_match_sequence_len_starred,
    r#"
from typing import assert_type
def test(x: tuple[int, ...] | tuple[int, *tuple[int, ...], int] | tuple[int, int, int]) -> None:
    match x:
        case [first, second, third, *middle, last]:
            # tuple[int, int, int] is narrowed away because the case requires least 4 elements
            assert_type(x, tuple[int, ...] | tuple[int, *tuple[int, ...], int])
"#,
);

testcase!(
    bug = "we don't narrow attributes in a positional pattern",
    test_match_class_union,
    r#"
from typing import assert_type, Literal

class Foo:
    x: int
    y: str
    __match_args__ = ("x", "y")

class Bar:
    x: str
    __match_args__ = ("x",)

def test(x: Foo | Bar) -> None:
    match x:
        case Foo(1, "a"):
            # we should narrow x.x and x.y to literals
            assert_type(x, Foo)
            assert_type(x.x, int)
            assert_type(x.y, str)
        case Foo(a, b):
            assert_type(x, Foo)
            assert_type(a, int)
            assert_type(b, str)
        case Foo(x = b, y = a):
            assert_type(x, Foo)
            assert_type(a, str)
            assert_type(b, int)
        case Foo(x = 1, y = ""):
            assert_type(x, Foo)
            assert_type(x.x, Literal[1])
            assert_type(x.y, Literal[""])
        case Bar("bar"):
            assert_type(x, Bar)
            assert_type(x.x, str)  # we want to narrow this to Literal["bar"]
        case Bar(a) as b:
            assert_type(x, Foo | Bar)
            assert_type(b, Bar)
            assert_type(a, str)
            assert_type(b, Bar)
"#,
);

testcase!(
    test_match_sequence_concrete,
    r#"
from typing import assert_type, Never

def foo(x: tuple[int, str, bool, int]) -> None:
    match x:
        case [bool(), b, c, d]:
            assert_type(x[0], bool)
            assert_type(b, str)
            assert_type(c, bool)
            assert_type(d, int)
        case [a, *rest]:
            assert_type(a, int)
            assert_type(rest, list[str | bool | int])
        case [a, *middle, b]:
            assert_type(a, int)
            assert_type(b, int)
            assert_type(middle, list[str | bool])
        case [a, b, c, d, e]:
            assert_type(x, Never)
        case [a, b, *middle, c, d]:
            assert_type(a, int)
            assert_type(b, str)
            assert_type(c, bool)
            assert_type(d, int)
            assert_type(middle, list[Never])
        case [*first, c, d]:
            assert_type(first, list[int | str])
            assert_type(c, bool)
            assert_type(d, int)
"#,
);

testcase!(
    test_match_sequence_unbounded,
    r#"
from typing import assert_type, Never

def foo(x: list[int]) -> None:
    match x:
        case []:
            pass
        case [a]:
            assert_type(a, int)
        case [a, b, c]:
            assert_type(a, int)
            assert_type(b, int)
            assert_type(c, int)
        case [a, *rest]:
            assert_type(a, int)
            assert_type(rest, list[int])
        case [a, *middle, b]:
            assert_type(a, int)
            assert_type(b, int)
            assert_type(middle, list[int])
        case [*first, a]:
            assert_type(first, list[int])
            assert_type(a, int)
        case [*all]:
            assert_type(all, list[int])
"#,
);

testcase!(
    test_match_or,
    r#"
from typing import assert_type

x: list[int] = [1, 2, 3]

match x:
    case [a] | a: # E: name capture `a` makes remaining patterns unreachable
        assert_type(a, list[int] | int)
    case [b] | _:
        assert_type(b, int)

match x:
    case _ | _:  # E: Only the last subpattern in MatchOr may be irrefutable
        pass
"#,
);

testcase!(
    test_crashing_match,
    r#"
match []:
    case [[1]]:
        pass
    case _:
        pass
"#,
);

testcase!(
    test_match_narrow_generic,
    r#"
from typing import assert_type
class C:
    x: list[int] | None

    def test(self):
        x = self.x
        match x:
            case list():
                assert_type(x, list[int])

    def test2(self):
        match self.x:
            case list():
                assert_type(self.x, list[int])
"#,
);

testcase!(
    test_error_in_test_expr,
    r#"
def f(x: None):
    if x.nonsense:  # E: Object of class `NoneType` has no attribute `nonsense`
        pass
    while x['nonsense']:  # E: `None` is not subscriptable
        pass
    "#,
);

// Regression test for a crash
testcase!(
    test_ternary_and_or,
    r#"
def f(x: bool, y: int):
    return 0 if x else (y or 1)
    "#,
);

fn loop_export_env() -> TestEnv {
    TestEnv::one(
        "imported",
        r#"
exported = None

for _ in []:
    ignored = 1
"#,
    )
}

testcase!(
    test_loop_export,
    loop_export_env(),
    r#"
import imported
from typing import assert_type

assert_type(imported.exported, None)
"#,
);

testcase!(
    test_loop_increment,
    r#"
from typing import assert_type, Literal

def f(cond: bool):
    n = 1
    while cond:
        n += 1
    assert_type(n, int)
"#,
);

testcase!(
    test_loop_test_and_increment,
    r#"
from typing import assert_type, Literal

def f(cond: bool):
    n = 1
    while n < 10:
        n += 1
    assert_type(n, int)
"#,
);

testcase!(
    bug = "We don't properly restrict loop checks to the current scope",
    test_nested_loop_increment,
    r#"
from typing import assert_type, Literal
def f_toplevel(cond: bool):
    n = "n"
    if cond:
        n = 1
    else:
        n = 1.5
    while cond:
        n += 1
    assert_type(n, float | int)
while True:
    def f_in_loop(cond: bool):
        n = "n"
        if cond:
            n = 1
        else:
            n = 1.5
        while cond:
            n += 1
        assert_type(n, float | int)
"#,
);

testcase!(
    test_loop_test_and_increment_return,
    r#"
from typing import assert_type, Literal

def f(cond: bool):
    n = 1
    while cond:
        n += 1
    return n

assert_type(f(True), int)
"#,
);

testcase!(
    test_nested_loops_simple,
    r#"
def f(cond1: bool, cond2: bool):
    n = 0
    while cond1:
        while cond2:
            n += 1
"#,
);

testcase!(
    test_nested_loops_return,
    r#"
from typing import assert_type, Literal

def f(cond1: bool, cond2: bool):
    n = 0
    while cond1:
        while cond2:
            n += 1
    return n

assert_type(f(True, True), int)
"#,
);

testcase!(
    test_augassign_in_loop_simple,
    r#"
def f(args, cond):
    n = 0
    for arg in args:
        if cond:
            n += 1
"#,
);

testcase!(
    test_augassign_in_loop_return,
    r#"
from typing import assert_type, Literal

def f(args, cond):
    n = 0
    for arg in args:
        if cond:
            n += 1
    return n

assert_type(f([1, 2, 3], True), int)
"#,
);

testcase!(
    test_loops_and_ifs_galore,
    r#"
from typing import assert_type, Literal

def f(cond1: bool, cond2: bool, cond3: bool, cond4: bool):
    i = 0
    while cond1:
        if cond2:
            if cond3:
                pass
            if cond4:
                i += 1
    return i

assert_type(f(True, True, True, True), int)
"#,
);

testcase!(
    test_loop_defaulting,
    r#"
# From https://github.com/facebook/pyrefly/issues/104
from typing import assert_type
class Foo:
    pass

def rebase(parent: Foo | int) -> Foo: ...

def test(b: bool, x: Foo) -> None:
    while b:
        x = rebase(x)
    assert_type(x, Foo)
"#,
);

testcase!(
    test_loop_enumerate,
    r#"
# From https://github.com/facebook/pyrefly/issues/267
def foo() -> list[int]:
    results: list[int] = [1, 2, 3]
    for i, x in enumerate(results):
        results[i] = x * 10
    return results
"#,
);

testcase!(
    test_if_which_exits,
    r#"
def foo(val: int | None, b: bool) -> int:
    if val is None:
        if b:
            return 1
        else:
            return 2
    return val
"#,
);

testcase!(
    test_shortcuit_or_after_flow,
    r#"
bar: str = "bar"

def func():
    foo: str | None = None

    for x in []:
        for y in []:
            pass

    baz: str = foo or bar
"#,
);

testcase!(
    test_export_not_in_flow,
    r#"
if 0.1:
    vari = "test"
    raise SystemExit
"#,
);

testcase!(
    test_assert_not_in_flow,
    r#"
from typing import assert_type, Literal
if 0.1:
    vari = "test"
    raise SystemExit
assert_type(vari, Literal["test"]) # E: `vari` is uninitialized
"#,
);

testcase!(
    test_assert_false_terminates_flow,
    r#"
def test1() -> int:
    assert False
def test2() -> int:  # E: Function declared to return `int` but is missing an explicit `return`
    assert True
    "#,
);

testcase!(
    bug = "Merge flow is lax about possibly-undefined locals, so we don't catch that `z` may be uninitialized.",
    test_named_inside_boolean_op,
    r#"
from typing import assert_type, Literal
b: bool = True
y = 5
x0 = True or (y := b) and False
assert_type(y, Literal[5, True])  # this is as expected
x0 = True or (z := b) and False
assert_type(z, bool)  # here, we did not catch that `z` may not be initialized
"#,
);

testcase!(
    test_loop_nested_binding,
    r#"
# This used to fail, thinking the type was Never
def f():
    class X:
        pass

    while True:
        z = "" if True else ""
        break
    else:
        exit(1)

    x: X
"#,
);

testcase!(
    test_loop_fails_to_reveal,
    r#"
# This used to get confused by what reveal_type is
from typing import *

x = 1

while True:
    reveal_type(x) # E: revealed type: Literal[1]
    break
else:
    exit(1)
"#,
);

testcase!(
    test_redundant_condition_func,
    r#"
def foo() -> bool: ...

if foo:  # E: Function object `foo` used as condition
    ...
while foo:  # E: Function object `foo` used as condition
    ...
[x for x in range(42) if foo]  # E: Function object `foo` used as condition
    "#,
);

testcase!(
    test_redundant_condition_class,
    r#"
class Foo:
    def __bool__(self) -> bool: ...

if Foo:  # E: Class name `Foo` used as condition
    ...
while Foo:  # E: Class name `Foo` used as condition
    ...
[x for x in range(42) if Foo]  # E: Class name `Foo` used as condition
    "#,
);

testcase!(
    test_redundant_condition_int,
    r#"
if 42:  # E: Integer literal used as condition. It's equivalent to `True`
    ...
while 0:  # E: Integer literal used as condition. It's equivalent to `False`
    ...
[x for x in range(42) if 42]  # E: Integer literal used as condition
    "#,
);

testcase!(
    test_redundant_condition_str_bytes,
    r#"
if "test":  # E: String literal used as condition. It's equivalent to `True`
    ...
while "":  # E: String literal used as condition. It's equivalent to `False`
    ...
[x for x in range(42) if b"test"]  # E: Bytes literal used as condition
    "#,
);

testcase!(
    test_redundant_condition_enum,
    r#"
import enum
class E(enum.Enum):
    A = 1
    B = 2
    C = 3
if E.A:  # E: Enum literal `E.A` used as condition
    ...
while E.B:  # E: Enum literal `E.B` used as condition
    ...
[x for x in range(42) if E.C]  # E: Enum literal `E.C` used as condition
    "#,
);

testcase!(
    crash_no_try_type,
    r#"
# Used to crash, https://github.com/facebook/pyrefly/issues/766
try:
    pass
except as r: # E: Parse error: Expected one or more exception types
    pass
"#,
);

testcase!(
    bug = "Loop recursion is causing problems, see https://github.com/facebook/pyrefly/issues/778",
    loop_with_sized_operation,
    r#"
intList: list[int] = [5, 6, 7, 8]
for j in [1, 2, 3, 4]:
    for i in range(len(intList)):  # E: `Sized | list[int]` is not assignable to `list[int]` (caused by inconsistent types when breaking cycles)
        intList[i] *= 42
print([value for value in intList])  # E: Type `Sized` is not iterable
"#,
);
