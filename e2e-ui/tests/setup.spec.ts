import { test, expect, type Page } from "@playwright/test";

test.beforeEach(async ({ page }) => {
    await page.goto("/admin?token=12345");
});
test.describe("Setup Criteria", () => {
    test("Code", async ({ page, browserName }) => {
        await createCriteria(page, `Code-${browserName}`);
    });
    test("Style", async ({ page, browserName }) => {
        await createCriteria(page, `Style-${browserName}`);
    });
});
test.describe("Setup Candidate", () => {
    test("Doe", async ({ page, browserName }) => {
        await createUser(page, `Doe-${browserName}`, "Candidate");
    });
    test("Muster", async ({ page, browserName }) => {
        await createUser(page, `Muster-${browserName}`, "Candidate");
    });
    test("Rust", async ({ page, browserName }) => {
        await createUser(page, `Rust-${browserName}`, "Candidate");
    });
    test("Java", async ({ page, browserName }) => {
        await createUser(page, `Java-${browserName}`, "Candidate");
    });
});
test.describe("Setup Voter", () => {
    test("John", async ({ page, browserName }) => {
        await createUser(page, `John-${browserName}`, "Voter");
    });
    test("user3", async ({ page, browserName }) => {
        await createUser(page, `user3-${browserName}`, "Voter");
    });
});
async function createCriteria(page: Page, name: String): void {
    await page.locator("#criteria-name").fill(name);
    await page.getByLabel("Minimum points").selectOption("1");
    await page.getByLabel("Maximum points").selectOption("10");
    await page.getByLabel("Weight in percentage").selectOption("5");
    await page.getByRole("button", { name: "Create criteria" }).click();
}
async function createUser(page: Page, name: String, type: String): void {
    await page.locator("#user-name").fill(name);
    await page.getByLabel("User type").selectOption(type);
    await page.getByRole("button", { name: "Create user" }).click();
}
