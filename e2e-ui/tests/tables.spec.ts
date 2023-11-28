import { test, expect, type Page } from "@playwright/test";
test.describe("tables", () => {
    test("overall", async ({ page }) => {
        await testTable(
            page,
            "/ballots/language",
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
                "Rust",
                "user3",
                "1",
                "10",
                "2",
                "10",
                "1",
                "cookie cool!",
                /\d{2}\.\d{2}\.\d{4}.*\(\d{2}:\d{2}\)/,
            ],
        );
    });
    test("by user", async ({ page }) => {
        await testTable(
            page,
            "/ballots/language/voters/user3",
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
                "Rust",
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

    test("by candidate", async ({ page }) => {
        await testTable(
            page,
            "/ballots/language/candidates/Rust",
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
                "user3",
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

    test("by sort mean", async ({ page }) => {
        await testTable(
            page,
            "/ballots/language/candidates/Rust?sort=mean",
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
                "user3",
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
    await expect(page.locator("th").first()).toHaveCSS(
        "text-transform",
        "uppercase",
    );
    await testTableRowByTag(page, "th", expectedHeaders);
    await testTableRowByTag(page, "tr", expectedRows, 1);
}

async function testTableRowByTag(
    page: Page,
    tag: "td" | "th",
    expected: string[],
    nth = 0,
): void {
    expect(page.locator(tag).nth(nth)).toContainText(expected);
}
