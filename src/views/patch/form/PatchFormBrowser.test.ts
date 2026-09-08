import { open as openDialog } from "@tauri-apps/plugin-dialog";
import userEvent from "@testing-library/user-event";
import { fireEvent, render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useFixtureBrowser } from "@/stores/fixture-browser.ts";
import { usePatchForm } from "@/stores/patch/patch-form.ts";
import { useSettings } from "@/stores/settings.ts";

import PatchFormBrowser from "./PatchFormBrowser.vue";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

const mockOpenDialog = vi.mocked(openDialog);

describe("PatchFormBrowser.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render search input and toolbar buttons", () => {
    const browser = useFixtureBrowser();
    browser.isLoading = false;

    render(PatchFormBrowser);

    expect(screen.getByPlaceholderText("Search GDTF & generic fixtures...")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import GDTF" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "GDTF-Share Settings" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Refresh fixtures" })).toBeInTheDocument();
  });

  it("should render clear button and reset when clicked", async () => {
    const user = userEvent.setup();
    const browser = useFixtureBrowser();
    browser.searchQuery = "SlimPAR";

    render(PatchFormBrowser);

    const patchForm = usePatchForm();
    const clearButton = screen.getByRole("button", { name: "Clear" });
    await user.click(clearButton);

    expect(browser.clearSearch).toHaveBeenCalled();
    expect(patchForm.reset).toHaveBeenCalled();
  });

  it("should call browser.refresh when refresh button is clicked", async () => {
    const user = userEvent.setup();
    const browser = useFixtureBrowser();
    browser.isLoading = false;

    render(PatchFormBrowser);

    const refreshButton = screen.getByRole("button", { name: "Refresh fixtures" });
    await user.click(refreshButton);

    expect(browser.refresh).toHaveBeenCalled();
  });

  it("should open Settings modal when GDTF-Share settings button is clicked", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    settings.isModalOpen = false;

    render(PatchFormBrowser);

    const accountButton = screen.getByRole("button", { name: "GDTF-Share Settings" });
    await user.click(accountButton);

    expect(settings.isModalOpen).toBe(true);
  });

  it("should import GDTF file when import button is clicked", async () => {
    const user = userEvent.setup();
    const browser = useFixtureBrowser();
    mockOpenDialog.mockResolvedValue("/path/to/custom.gdtf");

    render(PatchFormBrowser);

    const importButton = screen.getByRole("button", { name: "Import GDTF" });
    await user.click(importButton);

    expect(mockOpenDialog).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [{ name: "GDTF Fixture", extensions: ["gdtf", "zip"] }],
    });
    expect(browser.importGdtf).toHaveBeenCalledWith("/path/to/custom.gdtf");
  });

  it("should render manufacturer accordion and allow selecting fixture mode", async () => {
    const user = userEvent.setup();
    const browser = useFixtureBrowser();
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

  it("should render empty state when no fixtures match", () => {
    const browser = useFixtureBrowser();
    browser.isLoading = false;
    browser.searchQuery = "Nonexistent";
    browser.filteredManufacturers = {};

    render(PatchFormBrowser);

    expect(screen.getByText('No fixtures found matching "Nonexistent"')).toBeInTheDocument();
  });
});
