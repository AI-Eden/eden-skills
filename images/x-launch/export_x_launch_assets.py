"""
Export X launch card HTML files to PNGs using Playwright.

Outputs:
  - product-hook-card.png
  - pain-contrast-card.png
  - proof-status-card.png
"""

import asyncio
from pathlib import Path

from playwright.async_api import async_playwright

ROOT = Path(__file__).parent
HTML_FILES = [
    ROOT / "product-hook-card.html",
    ROOT / "pain-contrast-card.html",
    ROOT / "proof-status-card.html",
]

HEADLESS_SHELL = None
for candidate in Path("/tmp").rglob("chrome-headless-shell"):
    HEADLESS_SHELL = str(candidate)
    break


async def export_file(page, html_path: Path) -> None:
    png_path = html_path.with_suffix(".png")
    await page.goto(f"file://{html_path.resolve()}")
    await page.wait_for_load_state("networkidle")
    await page.evaluate("() => document.fonts.ready")
    await page.wait_for_timeout(350)
    card = page.locator(".card")
    await card.screenshot(path=str(png_path), type="png")
    size_kb = png_path.stat().st_size / 1024
    print(f"✓ {png_path.name} ({size_kb:.0f} KB)")


async def main() -> None:
    async with async_playwright() as playwright:
        launch_kwargs = {
            "headless": True,
            "args": [
                "--no-sandbox",
                "--disable-setuid-sandbox",
                "--disable-dev-shm-usage",
                "--use-gl=swiftshader",
            ],
        }
        if HEADLESS_SHELL:
            launch_kwargs["executable_path"] = HEADLESS_SHELL

        browser = await playwright.chromium.launch(**launch_kwargs)
        context = await browser.new_context(
            viewport={"width": 1760, "height": 1060},
            device_scale_factor=2,
        )
        page = await context.new_page()

        for html_path in HTML_FILES:
            await export_file(page, html_path)

        await browser.close()


if __name__ == "__main__":
    asyncio.run(main())
