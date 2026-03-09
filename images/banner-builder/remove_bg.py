"""
Logo background removal via luminosity masking.

Strategy: for a glowing-orb logo on a near-black background, the
alpha of each pixel should equal how "bright / luminous" that pixel is
relative to the dark background.

  alpha = clip( (max(R,G,B) - bg_floor) / (glow_ceil - bg_floor), 0, 1 ) ^ gamma

  bg_floor   ≈ peak brightness of pure-background pixels (~25 for dark navy)
  glow_ceil  ≈ brightness where we want fully-opaque (~90)
  gamma      ≈ 0.65  → softens the ramp so mid-tones keep more opacity
"""

from pathlib import Path

import numpy as np
from PIL import Image

# ── paths ──────────────────────────────────────────────────────────────
SRC = Path(__file__).parent.parent / "eden-skills-logo.png"
DST = Path(__file__).parent.parent / "eden-skills-logo-transparent.png"

# ── parameters (tune here) ─────────────────────────────────────────────
BG_FLOOR = 40  # brightness below this → fully transparent
GLOW_CEIL = 120  # brightness above this → fully opaque
GAMMA = 1.80  # exponent > 1 steepens the ramp: dark areas collapse to 0 faster

# ── load ───────────────────────────────────────────────────────────────
img = Image.open(SRC).convert("RGBA")
data = np.array(img, dtype=np.float32)

R, G, B = data[..., 0], data[..., 1], data[..., 2]

# "max-channel" brightness is the best proxy for glow intensity
# (it preserves saturated colors like cyan/green fully)
max_ch = np.maximum(np.maximum(R, G), B)  # [0, 255]

# Ramp: 0 at BG_FLOOR, 1 at GLOW_CEIL
ramp = np.clip((max_ch - BG_FLOOR) / (GLOW_CEIL - BG_FLOOR), 0.0, 1.0)

# Apply gamma curve – values < 1 make the ramp more "convex" so
# dim glows stay more opaque rather than fading out too early
alpha_f = ramp**GAMMA  # [0, 1]

# ── write back alpha channel ───────────────────────────────────────────
data[..., 3] = np.clip(alpha_f * 255, 0, 255).astype(np.uint8)

result = Image.fromarray(data.astype(np.uint8), "RGBA")
result.save(DST, optimize=True)

# ── report ─────────────────────────────────────────────────────────────
fully_transparent = int((data[..., 3] == 0).sum())
total = data.shape[0] * data.shape[1]
print(f"Saved  → {DST}")
print(f"Size   → {result.size[0]} × {result.size[1]} px")
print(
    f"Alpha=0 pixels: {fully_transparent}/{total}  ({100 * fully_transparent / total:.1f}%)"
)
