# rizzoo template — mode, polarity suffixes, and conditionals.
# Rendered via -r; output at example/output/conditionals.txt.
# Run with --mode light to see the light-mode variant.

# === MODE & POLARITY SUFFIXES ===
# Bare vars resolve to the current mode's roles.
# _dark / _light suffixes are always available regardless of mode.
mode:           {{ mode }}
primary_bare:   {{ primary }}
primary_dark:   {{ primary_dark }}
primary_light:  {{ primary_light }}

# === CONDITIONALS (ternary) ===
# condition ? if_true : if_false
# Conditions: bare var (truthy if set+non-empty), !var (falsey), == (literal eq)

# Eq condition — picks branch based on mode variable:
mode_label:     {{ mode == dark ? "dark" : "light" }}

# Truthiness — uses custom_accent if provided, else falls back to primary:
custom_or_fall: {{ custom_accent ? custom_accent : primary }}

# Negated truthiness — uses primary if custom_highlight is NOT set:
negated_fall:   {{ !custom_highlight ? primary : custom_highlight }}

# Filter inside a branch — darken light primary when in dark mode:
dark_branch_f:  {{ mode == dark ? primary:darken(0.2) : primary_light:lighten(0.1) }}