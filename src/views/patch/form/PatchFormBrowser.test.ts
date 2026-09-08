import userEvent from "@testing-library/user-event";
import { fireEvent, render, screen } from "@testing-library/vue";
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

  it("should call browser.refresh when refresh button is clicked", async () => {
    const user = userEvent.setup();
    const browser = useFixtureBrowser();
    browser.isInstalled = true;
    browser.isLoading = false;

    render(PatchFormBrowser);

    const refreshButton = screen.getByRole("button", { name: "Refresh fixtures" });
    expect(refreshButton).toBeInTheDocument();

    await user.click(refreshButton);
    expect(browser.refresh).toHaveBeenCalled();
  });

  it("should render manufacturer accordion and allow selecting fixture mode", async () => {
    const user = userEvent.setup();
    const browser = useFixtureBrowser();
    browser.isInstalled = true;
    browser.isLoading = false;
    browser.searchQuery = "SlimPAR";
    const fixtures = {
      chauvet: {
        name: "Chauvet DJ",
        fixtures: {
          "slimpar-64": {
            name: "SlimPAR 64",
            fixtureKey: "slimpar-64",
            manufacturerKey: "chauvet",
            categories: ["Color Changer"],
            modes: [{ name: "7 Channel", channels: ["Red", "Green", "Blue"] }],
          },
        },
      },
    };
    browser.manufacturers = fixtures;
    browser.filteredManufacturers = fixtures;

    render(PatchFormBrowser);

    expect(screen.getByText("Chauvet DJ")).toBeInTheDocument();
    const fixtureSummary = screen.getByText("SlimPAR 64");
    expect(fixtureSummary).toBeInTheDocument();

    const details = fixtureSummary.closest("details")!;
    details.open = true;
    await fireEvent(details, new Event("toggle"));

    const patchForm = usePatchForm();
    const modeButton = screen.getByRole("button", { name: /7 Channel/i });
    await user.click(modeButton);

    expect(patchForm.select).toHaveBeenCalledWith("chauvet", "slimpar-64", 0);
  });
});
