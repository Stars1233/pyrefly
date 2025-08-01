/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::testcase;

testcase!(
    test_fstring_literal,
    r#"
from typing import assert_type, Literal, LiteralString
x0 = f"abc"
assert_type(x0, Literal["abc"])

x1 = f"abc{x0}"
assert_type(x1, LiteralString)

x2 = f"abc" "def"
assert_type(x2, Literal["abcdef"])

x3 = f"abc" f"def"
assert_type(x3, Literal["abcdef"])

x4 = "abc" f"def"
assert_type(x4, Literal["abcdef"])

x5 = "abc" f"def{x0}g" "hij" f"klm"
assert_type(x5, LiteralString)
"#,
);

testcase!(
    test_invalid_literal,
    r#"
from typing import Literal
x = 1
y: Literal[x]  # E: Expected a type form
"#,
);

testcase!(
    test_large_int_literal,
    r#"
from typing import assert_type, Literal
x = 1
y = 0xFFFFFFFFFFFFFFFFFF
assert_type(x, Literal[1])
assert_type(y, Literal[4722366482869645213695])
"#,
);

testcase!(
    test_large_int_type,
    r#"
from typing import Literal
x: Literal[0xFFFFFFFFFFFFFFFFFF]
"#,
);

testcase!(
    test_generic_create_literal,
    r#"
from typing import assert_type, Literal

class Foo[T]:
    def __init__(self, x: T) -> None: ...

x: Literal[42] = 42
assert_type(Foo(x), Foo[int])
"#,
);

testcase!(
    test_generic_get_literal,
    r#"
from typing import assert_type, Literal

class Foo[T]:
    def get(self) -> T: ...

def test(x: Foo[Literal[42]]) -> None:
    assert_type(x.get(), Literal[42])
"#,
);

testcase!(
    test_literal_string_after_if,
    r#"
from typing import Literal

if True:
    pass

x: Literal["little", "big"] = "big"
"#,
);

testcase!(
    test_literal_none,
    r#"
from typing import Literal
Literal[None]
    "#,
);

testcase!(
    test_literal_alias,
    r#"
from typing import Literal as L
x: L["foo"] = "foo"
"#,
);

testcase!(
    test_literal_string_infer,
    r#"
from typing import LiteralString, assert_type
def f(x: LiteralString):
    assert_type(["foo"], list[str])
    assert_type([x], list[LiteralString])
    xs: list[str] = [x]
"#,
);

testcase!(
    test_index_literal,
    r#"
from typing import assert_type

def foo(x):
    assert_type("Magic"[0], str)
    assert_type("Magic"[3:4], str)
"#,
);

testcase!(
    test_index_bool,
    r#"
from typing import assert_type, Literal
t = ("a", "b")
assert_type(t[False], Literal["a"])
assert_type(t[True], Literal["b"])

"#,
);

testcase!(
    test_literal_nesting,
    r#"
from typing import Literal, assert_type

X = Literal["foo", "bar"]
Y = Literal["baz", None]
Z = Literal[X, Y]

def f(x: Z) -> None:
    assert_type(x, Literal["foo", "bar", "baz", None])
"#,
);

testcase!(
    test_literal_direct_nesting,
    r#"
from typing import Literal

good: Literal[Literal[Literal[1, 2, 3], "foo"], 5, None] = "foo"
bad: Literal[Literal, 3]  # E: Expected a type argument for `Literal`  # E: Invalid type inside literal, `Literal`
"#,
);

testcase!(
    test_literal_brackets,
    r#"
from typing import Literal
bad6: Literal[(1, "foo", "bar")]  # E: `Literal` arguments cannot be parenthesized
"#,
);

testcase!(
    test_literal_with_nothing,
    r#"
from typing import Literal
bad1: Literal # E: Expected a type argument for `Literal`
bad2: list[Literal]  # E: Expected a type argument for `Literal`
"#,
);

testcase!(
    test_literal_with_byte,
    r#"
from typing import assert_type, Literal
x = b"far"

assert_type(x[0], Literal[102])
x[3.14]  # E: Cannot index into `Literal[b'far']`
y: Literal[0] = 0
assert_type(x[y], Literal[102])

# Negative index case
assert_type(x[-1], Literal[114])
x[-6.28]  # E: Cannot index into `Literal[b'far']`

# The `bytes` type is correct, but ideally we would understand
# literal slices and be able to give back the literal bytes.
assert_type(x[0:1], Literal[b"f"])  # E: assert_type(bytes, Literal[b'f'])

# Non-literal integers give back an `int` (one byte)
i: int = 42
assert_type(x[i], int)
"#,
);

testcase!(
    test_bad_literal,
    r#"
# This used to crash, see https://github.com/facebook/pyrefly/issues/453
0x_fffffffffffffffff
1_23
"#,
);
