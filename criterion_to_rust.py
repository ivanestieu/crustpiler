#!/usr/bin/env python3

import re
import sys
from pathlib import Path

def normalize_eq(lhs: str, rhs: str) -> tuple[str, str]:
    """
    Normalize the order of the left-hand side and right-hand side of an
    equality assertion. If one side contains a function call and the other
    does not, the side with the function call will be placed on the
    left-hand side. If both sides contain function calls or neither side
    contains a function call, the order will remain unchanged.
    """
    func_call_re = re.compile(r"\b[A-Za-z_]\w*\s*\(")

    lhs_has_call = bool(func_call_re.search(lhs))
    rhs_has_call = bool(func_call_re.search(rhs))

    if lhs_has_call and not rhs_has_call:
        return lhs, rhs

    if rhs_has_call and not lhs_has_call:
        return rhs, lhs

    return lhs, rhs

def replace_assert_eq(match : re.Match) -> str:
    """
    Replace Criterion's cr_assert_eq and cr_expect_eq with Rust's assert_eq! macro.
    """
    lhs = match.group(1).strip()
    rhs = match.group(2).strip()

    lhs, rhs = normalize_eq(lhs, rhs)

    return f"assert_eq!({lhs}, {rhs});"

def convert_asserts(body: str) -> str:
    """
    Convert a few common Criterion assertions to Rust assertions.
    """

    replacements = [
        (
            r"cr_assert_eq\s*\(\s*(.+?)\s*,\s*(.+?)\s*\)\s*;",
            replace_assert_eq,
        ),
        (
            r"cr_expect_eq\s*\(\s*(.+?)\s*,\s*(.+?)\s*\)\s*;",
            replace_assert_eq,
        ),
        (
            r"cr_assert_neq\s*\(\s*(.+?)\s*,\s*(.+?)\s*\)\s*;",
            r"assert_ne!(\1, \2);",
        ),
        (
            r"cr_assert\s*\(\s*(.+?)\s*\)\s*;",
            r"assert!(\1);",
        ),
    ]

    for pattern, replacement in replacements:
        body = re.sub(pattern, replacement, body, flags=re.DOTALL)

    return body

def extract_timeout(source: str):
    """
    Extract the timeout value from the source code, if present.
    """

    pattern = re.compile(
        r"TestSuite\s*\(\s*([A-Za-z_]\w*)\s*(?:,\s*(.*?)\s*)?\)\s*;",
    re.DOTALL,
    )

    suite_match = pattern.search(source)

    if suite_match:
        options = suite_match.group(2) or ""

        timeout_match = re.search(
            r"\.timeout\s*=\s*(\d+)",
            options,
        )

        if timeout_match:
            return int(timeout_match.group(1))
        
    return None

def extract_tests(source: str):
    """
    Extract Criterion tests of the form:

        Test(suite, name)
        {
            ...
        }
    """

    pattern = re.compile(
        r"Test\s*\(\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*\)\s*"
        r"\{(.*?)\}",
        re.DOTALL,
    )

    tests = []

    for suite, name, body in pattern.findall(source):
        body = convert_asserts(body.strip())

        tests.append(
            {
                "suite": suite,
                "name": name,
                "body": body,
            }
        )

    return tests

def extract_package_name(source: str):
    """
    Extract the package name from Cargo.toml source code.
    """
    find_name = re.search(r'name\s*=\s*"([^"]+)"', source)
    if not find_name:
        print("Error: Could not find package name in Cargo.toml")
        sys.exit(1)
    return find_name.group(1)

def add_ntest_dependency():
    """
    Add the ntest dependency to Cargo.toml if it is not already present.
    """
    cargo_toml_path = Path("Cargo.toml")
    cargo_source = cargo_toml_path.read_text(encoding="utf-8")

    if "ntest" not in cargo_source:
        if not "[dependencies]" in cargo_source:
            cargo_source += "\n[dependencies]\n"
        cargo_source += 'ntest = "0.9.5"\n'
        cargo_toml_path.write_text(cargo_source, encoding="utf-8")

def generate_rust(tests, package_name, timeout=None):
    """
    Generate Rust test code from the extracted tests.
    """
    out = []

    if timeout is not None:
        add_ntest_dependency()

    out.append("#[cfg(test)]")
    out.append(f"mod {package_name} {{")
    out.append(f"    use {package_name}::*;")
    out.append("")

    for test in tests:
        out.append("    #[test]")
        if timeout is not None:
            out.append(f"    #[ntest::timeout({timeout})]")
        out.append(f"    fn {test['name']}() {{")

        for line in test["body"].splitlines():
            out.append(f"        {line.rstrip()}")

        out.append("    }")
        out.append("")

    out.append("}")

    return "\n".join(out)


def main():
    """
    Main function to convert Criterion tests to Rust tests.
    """
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} test.c [Cargo.toml PATH]")
        sys.exit(1)

    cargo_source = Path("Cargo.toml" if not len(sys.argv) > 2 else sys.argv[2]).read_text(encoding="utf-8")
    source = Path(sys.argv[1]).read_text(encoding="utf-8")

    tests = extract_tests(source)
    timeout = extract_timeout(source)
    package_name = extract_package_name(cargo_source)
    rust = generate_rust(tests, package_name, timeout=timeout)

    print(rust)


if __name__ == "__main__":
    main()
