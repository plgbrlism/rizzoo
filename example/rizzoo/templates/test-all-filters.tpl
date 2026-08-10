# rizzoo template feature tour — every variable and filter in one file.
# Rendered via -r into the cache, then copied to example/output by -o.
#
# NOTE: there is no comment syntax in rizzoo templates — lines starting with
# '#' are plain text. Keep this header free of double braces.
#
# Context variables:
#   Material roles as hex:  primary, on_primary, surface, background,
#                           outline, error, tertiary, ... (all MaterialRoles)
#   Base16 palette:         base00 .. base15
#   Base16 array:           the colors array (see the for loop below)
#   Custom colors:          custom_accent, custom_highlight
#   Wallpaper path:         wallpaper
#   image is only available in the [wallpaper] command, not in templates.

# === PASSTHROUGH (vars already hold hex) ===
primary:        {{ primary }}
surface:        {{ surface }}
background:     {{ background }}
on_surface:     {{ on_surface }}

# === FORMAT FILTERS ===
hex_raw:        {{ primary:hex_raw }}
rgb:            {{ primary:rgb }}
rgba(0.5):      {{ primary:rgba(1.0) }}
hsl:            {{ primary:hsl }}
hsla:           {{ primary:hsla(1.0) }}
hue:            {{ primary:hue }}
saturation:     {{ primary:saturation }}
lightness:      {{ primary:lightness }}
r/g/b:          {{ primary:r }} {{ primary:g }} {{ primary:b }}

# === LIGHTEN / DARKEN / SATURATE / DESATURATE (default amount 0.1) ===
lighten(0.1):   {{ primary:lighten(0.1) }}
lighten(0.5):   {{ primary:lighten(0.5) }}
darken(0.1):    {{ primary:darken(0.1) }}
saturate(0.2):  {{ primary:saturate(0.2) }}
desaturate(0.2): {{ primary:desaturate(0.2) }}
auto_lightness: {{ primary:auto_lightness(0.1) }}

# === TRANSFORMS ===
invert:         {{ primary:invert }}
grayscale:      {{ primary:grayscale }}

# === BLEND / HARMONIZE / ENSURE_CONTRAST ===
# args: quoted hex strings, bare %vars (resolved from context), or numbers
blend %secondary (50/50):  {{ primary:blend(%secondary) }}
blend %secondary 0.5:      {{ primary:blend(%secondary, 0.5) }}
blend %secondary 0.3:      {{ primary:blend(%secondary, 0.3) }}
blend "#ff0000" 0.5:       {{ primary:blend("#ff0000", 0.5) }}
harmonize %tertiary:       {{ primary:harmonize(%tertiary) }}
ensure_contrast:           {{ on_surface:ensure_contrast(%surface, 4.5) }}

# === BASE16: FOR LOOP OVER THE COLORS ARRAY ===
colors (16 lines):
{{#for c in colors }}{{ c }}
{{/c }}

base00/base07/base08/base15:
{{ base00 }} {{ base07 }} {{ base08 }} {{ base15 }}

# === CUSTOM COLORS ===
custom_accent:    {{ custom_accent }}
custom_highlight: {{ custom_highlight }}

# === WALLPAPER PATH ===
wallpaper: {{ wallpaper }}
