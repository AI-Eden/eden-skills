"""
High-quality PNG export for banner.html
────────────────────────────────────────
Strategy
  • deviceScaleFactor=2  →  2400×800 output (2× retina)
  • waitForLoadState("networkidle")  →  all CDN fonts fetched
  • document.fonts.ready  →  browser has measured + rasterised glyphs
  • wait for logo <img> complete  →  2048×2048 source fully decoded
  • element.screenshot() on .banner-container  →  exact 1200×400 crop,
    no body padding, no outer shadow bleed
"""

import asyncio
from pathlib import Path

from playwright.async_api import async_playwright

BANNER_HTML = Path(__file__).parent.parent / "banner.html"
OUTPUT_PNG = Path(__file__).parent.parent / "banner-export.png"

# ── find the Chromium installed by npx playwright install ──────────────
# agent-browser stores it under /tmp/cursor-sandbox-cache/…
# playwright-python looks in its own cache (~/.cache/ms-playwright)
# We let playwright-python use its own channel discovery first; fall back
# to the headless-shell we know exists.
HEADLESS_SHELL = None
for candidate in Path("/tmp").rglob("chrome-headless-shell"):
    HEADLESS_SHELL = str(candidate)
    break


async def export():
    async with async_playwright() as pw:
        launch_kwargs = dict(
            headless=True,
            args=[
                "--no-sandbox",
                "--disable-setuid-sandbox",
                "--disable-dev-shm-usage",
                # ensure backdrop-filter renders in headless mode
                "--enable-features=VaapiVideoDecoder",
                "--use-gl=swiftshader",
            ],
        )
        if HEADLESS_SHELL:
            launch_kwargs["executable_path"] = HEADLESS_SHELL

        browser = await pw.chromium.launch(**launch_kwargs)

        ctx = await browser.new_context(
            device_scale_factor=2,  # 2× retina → 2400×800 px
            viewport={"width": 1400, "height": 600},
        )
        page = await ctx.new_page()

        # file:// URL so relative ./eden-skills-logo-transparent.png resolves
        await page.goto(f"file://{BANNER_HTML.resolve()}")

        # 1. wait for all network requests (fonts from CDN)
        await page.wait_for_load_state("networkidle")

        # 2. wait for web fonts to be measured and rasterised
        await page.evaluate("() => document.fonts.ready")

        # 3. wait for the logo <img> to finish decoding
        await page.evaluate("""
            () => {
                const img = document.querySelector('img.logo-img');
                if (!img) return Promise.resolve();
                if (img.complete) return Promise.resolve();
                return new Promise(r => { img.onload = r; img.onerror = r; });
            }
        """)

        # 4. one extra paint tick for any pending CSS transitions
        await page.wait_for_timeout(400)

        # 5. screenshot the banner element only
        banner = page.locator(".banner-container")
        await banner.screenshot(path=str(OUTPUT_PNG), type="png")

        await browser.close()

    size = OUTPUT_PNG.stat().st_size / 1024
    print(f"✓ Exported  →  {OUTPUT_PNG}")
    print(f"  File size  : {size:.0f} KB")
    print("  Canvas     : 1200×400 CSS px  ×  DPR 2  =  2400×800 physical px")


asyncio.run(export())
