import { test, expect, type Page } from "@playwright/test";
test.describe("tables", () => {
    test("overall", async ({ page, browserName }) => {
        await testTable(
            page,
            `/ballots/language-${browserName}`,
            [
                "voter",
                "candidate",
                "Code",
                "Style",
                "sum",
                "weighted",
                "mean",
                "note",
                "voted",
            ],
            [
                `user3-${browserName}`,
                `Java-${browserName}`,
                "1",
                "10",
                "2",
                "10",
                "1",
                /COOKIE\d{1} COOL!/,
                /\d{2}\.\d{2}\.\d{4}.*\(\d{2}:\d{2}\)/,
            ],
        );
    });
    test("by user", async ({ page, browserName }) => {
        await testTable(
            page,
            `/ballots/language-${browserName}/voters/user3-${browserName}`,
            [
                "candidate",
                "Code",
                "Style",
                "sum",
                "weighted",
                "mean",
                "note",
                "voted",
            ],
            [
                `Rust-${browserName}`,
                "1",
                "1",
                "2",
                "10",
                "1",
                "cookie cool!",
                /\d{2}\.\d{2}\.\d{4}.*\(\d{2}:\d{2}\)/,
            ],
        );
    });

    test("by candidate", async ({ page, browserName }) => {
        await testTable(
            page,
            `/ballots/language-${browserName}/candidates/Rust-${browserName}`,
            [
                "voter",
                "Code",
                "Style",
                "sum",
                "weighted sum",
                "mean",
                "note",
                "voted",
            ],
            [
                `user3-${browserName}`,
                "1",
                "1",
                "2",
                "10",
                "1",
                "cookie cool!",
                /\d{2}\.\d{2}\.\d{4}.*\(\d{2}:\d{2}\)/,
            ],
        );
    });

    test("by sort mean", async ({ page, browserName }) => {
        await testTable(
            page,
            `/ballots/language-${browserName}/candidates/Rust-${browserName}?sort=mean`,
            [
                "voter",
                "Code",
                "Style",
                "sum",
                "weighted",
                "mean",
                "note",
                "voted",
            ],
            [
                `user3-${browserName}`,
                "1",
                "1",
                "2",
                "10",
                "1",
                "cookie cool!",
                /\d{2}\.\d{2}\.\d{4}.*\(\d{2}:\d{2}\)/,
            ],
        );
    });
});

async function testTable(
    page: Page,
    url: string,
    expectedHeaders: string[],
    expectedRows: string[],
): void {
    await page.goto(url);
    await expectedHeaders.map(async (e, i) => {
        expect(await page.locator("th").nth(i)).toContainText(e);
    });
    await expectedRows.map(async (e, i) => {
        await expect(
            await page
                .locator("tr")
                .nth(i + 1)
                .locator("td"),
        ).toContainText(e);
    });
}
