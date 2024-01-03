import { test, expect, type Page } from "@playwright/test";
test.describe("Votings", () => {
    test("John", async ({ page, browserName }) => {
        await page.goto("/");
        await page.locator("#invite-code").fill("e1234");
        await page.getByText(`Language-${browserName}`).click();
        await testStyle(page, browserName);
        const url = page.url();
        await vote(page, browserName, [
            `John-${browserName}`,
            `Muster-${browserName}`,
            "3",
            "5",
            "cookie cool!",
        ]);
        await page.goto(url);
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
        await page.getByText(`Language-${browserName}`).click();
        await testStyle(page, browserName);
        const url = page.url();
        await vote(page, browserName, [
            `user3-${browserName}`,
            `Muster-${browserName}`,
            "10",
            "10",
            "cookie0 cool!",
        ]);
        await page.goto(url);
        await vote(page, browserName, [
            `user3-${browserName}`,
            `Java-${browserName}`,
            "8",
            "2",
            "cookie2 cool!",
        ]);
    });
});

async function testStyle(page: Page, browserName: string): void {
    const body = page.locator("body");
    await expect(body).toHaveClass(`language-${browserName}`);
    await expect(body).toHaveCSS("background-color", "rgb(16, 50, 61)");
    await expect(body).toHaveCSS("color", "rgb(240, 255, 0)");
    await testStylesForFormFields(page, "select");
    await testStylesForFormFields(page, "input");
}
async function vote(page: Page, browserName: string, values: string[]): void {
    const upperCasedInputs = values.map((v) => v.toUpperCase());
    const name = page.locator("#name");
    await name.selectOption(upperCasedInputs[0]);
    await expect(name).toHaveValue(values[0].toLowerCase());

    const canidate = page.locator("#candidate");
    await canidate.selectOption(upperCasedInputs[1]);
    await expect(canidate).toHaveValue("candidate_" + values[1].toLowerCase());

    const code = page.locator(`#Code-${browserName}`);
    await code.selectOption(upperCasedInputs[2]);
    await expect(code).toHaveValue(values[2]);

    const style = page.locator(`#Style-${browserName}`);
    await style.selectOption(upperCasedInputs[3]);
    await expect(style).toHaveValue(values[3]);

    const notes = page.locator("#notes");
    await notes.fill(upperCasedInputs[4]);
    await expect(notes).toHaveValue(upperCasedInputs[4]);
    await page.getByRole("button", { name: "😍 Submit 🥇 🥈 🥉" }).click();
}
async function testStylesForFormFields(page: Page, tag: string): void {
    const element = page.locator(tag).first();
    await expect(element).toHaveCSS("color", "rgb(19, 54, 61)");
    await expect(element).toHaveCSS("background-color", "rgb(37, 118, 61)");
}
