import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useFixtureBrowser } from "@/stores/fixture-browser.ts";
import { usePatchForm } from "@/stores/patch/patch-form.ts";

import PatchFormBrowser from "./PatchFormBrowser.vue";

describe("PatchFormBrowser.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render download prompt when fixtures are not installed", async () => {
    const user = userEvent.setup();
    const browser = useFixtureBrowser();
    browser.isInstalled = false;
    browser.isDownloading = false;

    render(PatchFormBrowser);

    expect(screen.getByText("Fixture Library Required")).toBeInTheDocument();
    const downloadButton = screen.getByRole("button", { name: /Download Fixtures/i });
    expect(downloadButton).toBeInTheDocument();

    await user.click(downloadButton);
    expect(browser.downloadFixtures).toHaveBeenCalled();
  });

  it("should render search input and clear button when fixtures are installed", async () => {
    const user = userEvent.setup();
    const browser = useFixtureBrowser();
    browser.isInstalled = true;
    browser.isLoading = false;

    render(PatchFormBrowser);

    const input = screen.getByPlaceholderText("Search fixtures...");
    expect(input).toBeInTheDocument();

    const patchForm = usePatchForm();

    const clearButton = screen.getByRole("button", { name: "Clear" });
    await user.click(clearButton);

    expect(browser.clearSearch).toHaveBeenCalled();
    expect(patchForm.reset).toHaveBeenCalled();
  });
});
