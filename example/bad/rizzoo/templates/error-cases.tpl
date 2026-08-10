# Intentional errors — every line below is broken.
# Rizzoo's `-r` aborts on the FIRST error and there is no --continue-on-error
# (matugen has one; rizzoo does not yet). See run-bad.sh.
#
# Each line is syntactically valid but semantically wrong, so it parses and
# fails at evaluation — good for exercising error messages:
#
#   undefined variable
{{ does_not_exist }}
#
#   unknown filter
{{ primary:unknown_filter }}
#
#   blend with a nonexistent target color
{{ primary:blend(%does_not_exist) }}
#
#   ensure_contrast with a missing ratio arg
{{ primary:ensure_contrast(%surface) }}
