import { test, expect, type Page } from "@playwright/test";
test.describe("Votings", () => {
    test("John", async ({ page }) => {
        await page.goto("/");
        await page.locator("#invite-code").fill("e1234");
        await page.getByText("language").click();
        await testStyle(page);
        await vote(page, browserName, [
            `John-${browserName}`,
            `Muster-${browserName}`,
            "3",
            "5",
            "cookie cool!",
        ]);
        await vote(page, browserName, [
            `John-${browserName}`,
            `Rust-${browserName}`,
            "2",
            "8",
            "cookie3 cool!",
        ]);
    });

    test("user3", async ({ page, browserName }) => {
        await page.goto("/");
        await page.locator("#invite-code").fill("e1234");
        await page.getByText("language").click();
        await testStyle(page);
        await vote(page, browserName, [
            `user3-${browserName}`,
            `Muster-${browserName}`,
            "1",
            "1",
            "cookie0 cool!",
        ]);
        await vote(page, browserName, [
            `user3-${browserName}`,
            `Java-${browserName}`,
            "8",
            "2",
            "cookie2 cool!",
        ]);
    });
});

async function testStyle(page: Page): void {
    const body = page.locator("body");
    await expect(body).toHaveClass("language");
    await expect(body).toHaveCSS("background-color", "rgb(16, 50, 61)");
    await expect(body).toHaveCSS("color", "rgb(240, 255, 0)");
    await testStylesForFormFields(page, "select");
    await testStylesForFormFields(page, "input");
}
async function vote(page: Page, browserName: string, values: string[]): void {
    const name = page.locator("#name");
    await name.selectOption(values[0]);
    await expect(name).toHaveValue(values[0]);

    const canidate = page.locator("#candidate");
    await canidate.selectOption(values[1]);
    await expect(canidate).toHaveValue("candidate_" + values[1]);

    const code = page.locator(`#Code--${browserName}`);
    await code.selectOption(values[2]);
    await expect(code).toHaveValue(values[2]);

    const style = page.locator(`#Style-${browserName}`);
    await style.selectOption(values[3]);
    await expect(style).toHaveValue(values[3]);

    const notes = page.locator("#notes");
    await notes.fill(values[4]);
    await expect(notes).toHaveValue(values[4]);
    await page.getByRole("button", { name: "😍 Absenden 🥇 🥈 🥉" }).click();
}
async function testStylesForFormFields(page: Page, tag: string): void {
    const element = page.locator(tag).first();
    await expect(element).toHaveCSS("color", "rgb(19, 54, 61)");
    await expect(element).toHaveCSS("background-color", "rgb(37, 118, 61)");
}
