import { test, expect, type Page } from "@playwright/test";
test.describe("Create voting", () => {
    test("Create voting", async ({ page, browserName }) => {
        await page.goto("/admin/votings?token=12345");
        const name = page.locator("#voting-id");
        await name.fill(`language-${browserName}`);
        await expect(name).toHaveValue(`language-${browserName}`);

        const code = page.getByLabel(
            `Code-${browserName} (Min:1) (Max:10) (W:5)`,
        );
        await code.check();
        await expect(code).toBeChecked();

        const style = page.getByLabel(
            `Style-${browserName} (Min:1) (Max:10) (W:5)`,
        );
        await style.check();
        await expect(style).toBeChecked();

        const doe = page.getByLabel(`Rust-${browserName}`);
        await doe.check();
        await expect(doe).toBeChecked();

        const rust = page.getByLabel(`Muster-${browserName}`);
        await rust.check();
        await expect(rust).toBeChecked();
        const java = page.getByLabel(`Java-${browserName}`);
        await java.check();
        await expect(java).toBeChecked();
        const endDate = page.locator("#ends");
        await endDate.fill("2030-11-11");
        await expect(endDate).toHaveValue("2030-11-11");

        const invite_code = page.locator("#voting-invite");
        await invite_code.fill("e1234");

        const colorFont = page.locator("#voting-color-font");
        await colorFont.fill("#f0ff00");
        await expect(colorFont).toHaveValue("#f0ff00");

        const colorBg = page.locator("#voting-color-bg");
        await colorBg.fill("#10323d");
        await expect(colorBg).toHaveValue("#10323d");

        const fieldFont = page.locator("#voting-field-font");
        await fieldFont.fill("#13363d");

        const fieldBg = page.locator("#voting-field-bg");
        await fieldBg.fill("#25763d");
        await expect(fieldBg).toHaveValue("#25763d");
        await page.getByRole("button", { name: "Create voting" }).click();
    });
});
