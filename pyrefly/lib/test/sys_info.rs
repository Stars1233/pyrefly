/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use pyrefly_python::sys_info::PythonVersion;

use crate::test::util::TestEnv;
use crate::testcase;

testcase!(
    test_sys_version,
    r#"
from typing import assert_type
import sys
if sys.version_info >= (3, 0):
    X = str
else:
    X = int
assert_type(X(), str)

if sys.version_info == (2, 7):
    Y = str
else:
    Y = int
assert_type(Y(), int)

if sys.version_info < (3, 0, 0):
    Z = str
else:
    Z = int
assert_type(Z(), int)
"#,
);

testcase!(
    test_class_under_version,
    r#"
from typing import assert_type
import sys
if sys.version_info >= (3, 0):
    class Bar:
        def magic(self) -> Foo:
            return Foo()
    class Foo: ...

    assert_type(Bar().magic(), Foo)
"#,
);

testcase!(
    test_bool_literals,
    r#"
from typing import assert_type
if True:
    X = str
else:
    X = int
assert_type(X(), str)

if False:
    Y = str
else:
    Y = int
assert_type(Y(), int)

if not(False):
    X = str
else:
    X = int
assert_type(X(), str)

if not(True):
    Y = str
else:
    Y = int
assert_type(Y(), int)
"#,
);

testcase!(
    test_typechecking_constant,
    r#"
import typing
import typing_extensions as te
from typing import TYPE_CHECKING, assert_type
if TYPE_CHECKING:
    X0 = str
else:
    X0 = int
assert_type(X0(), str)

if typing.TYPE_CHECKING:
    X1 = str
else:
    X1 = int
assert_type(X1(), str)

if not TYPE_CHECKING:
    Y0 = str
else:
    Y0 = int
assert_type(Y0(), int)

if not typing.TYPE_CHECKING:
    Y1 = str
else:
    Y1 = int
assert_type(Y1(), int)

if te.TYPE_CHECKING:
    Z1 = str
else:
    Z1 = bool
assert_type(Z1(), str)
"#,
);

testcase!(
    test_typechecking_with_pyrefly_constant,
    TestEnv::one("foo", "TYPE_CHECKING_WITH_PYREFLY: bool = False"),
    r#"
from typing import assert_type
from foo import TYPE_CHECKING_WITH_PYREFLY

if TYPE_CHECKING_WITH_PYREFLY:
    X0 = str
else:
    X0 = int
assert_type(X0(), str)

import foo
if foo.TYPE_CHECKING_WITH_PYREFLY:
    X1 = str
else:
    X1 = int
assert_type(X1(), str)
"#,
);

testcase!(
    test_platform,
    r#"
from typing import assert_type
import sys
if sys.platform == "linux":
    X = str
else:
    X = int
assert_type(X(), str)

if sys.platform.startswith("win"):
    Y = str
elif sys.platform.startswith("lin"):
    Y = int
else:
    Y = None
assert_type(Y(), int)
"#,
);

testcase!(
    test_sys_info_with_or,
    r#"
from typing import TYPE_CHECKING, Literal, assert_type
x = True

if x or TYPE_CHECKING:
    y = ""
else:
    y = 1

assert_type(y, Literal[''])
"#,
);

testcase!(
    test_python_3_14,
    TestEnv::new_with_version(PythonVersion::new(3, 14, 0)),
    "",
);

testcase!(
    test_python_3_13,
    TestEnv::new_with_version(PythonVersion::new(3, 13, 0)),
    "",
);

testcase!(
    test_python_3_12,
    TestEnv::new_with_version(PythonVersion::new(3, 12, 0)),
    "",
);

testcase!(
    test_python_3_11,
    TestEnv::new_with_version(PythonVersion::new(3, 11, 0)),
    "",
);

testcase!(
    test_python_3_10,
    TestEnv::new_with_version(PythonVersion::new(3, 10, 0)),
    "",
);

testcase!(
    test_python_3_9,
    TestEnv::new_with_version(PythonVersion::new(3, 9, 0)),
    "",
);

testcase!(
    test_python_3_8,
    TestEnv::new_with_version(PythonVersion::new(3, 8, 0)),
    "",
);

testcase!(
    test_python_3_7,
    TestEnv::new_with_version(PythonVersion::new(3, 7, 0)),
    "",
);
