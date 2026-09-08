import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useSettings } from "@/stores/settings.ts";

import SettingsModal from "./SettingsModal.vue";

describe("SettingsModal.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not render modal when isModalOpen is false", () => {
    const settings = useSettings();
    settings.isModalOpen = false;

    render(SettingsModal);

    expect(screen.queryByRole("heading", { name: /settings/i })).not.toBeInTheDocument();
  });

  it("renders settings options when isModalOpen is true", () => {
    const settings = useSettings();
    settings.isModalOpen = true;

    render(SettingsModal);

    expect(screen.getByRole("heading", { name: /settings/i })).toBeInTheDocument();
    expect(screen.getByText("Global")).toBeInTheDocument();
    expect(screen.getByText("Live")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Dark theme" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Light theme" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "System theme" })).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Show button labels" })).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Show section titles" })).toBeInTheDocument();
  });

  it("changes theme when clicking theme buttons", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    settings.isModalOpen = true;

    render(SettingsModal);

    await user.click(screen.getByRole("button", { name: "Dark theme" }));
    expect(settings.theme).toBe("dark");

    await user.click(screen.getByRole("button", { name: "Light theme" }));
    expect(settings.theme).toBe("light");

    await user.click(screen.getByRole("button", { name: "System theme" }));
    expect(settings.theme).toBe("system");
  });

  it("toggles show button labels when clicking checkbox", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    settings.isModalOpen = true;
    settings.live.showButtonLabels = true;

    render(SettingsModal);

    const checkbox = screen.getByRole("checkbox", { name: "Show button labels" });
    await user.click(checkbox);

    expect(settings.live.showButtonLabels).toBe(false);
  });

  it("toggles show section titles when clicking checkbox", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    settings.isModalOpen = true;
    settings.live.showSectionTitles = true;

    render(SettingsModal);

    const checkbox = screen.getByRole("checkbox", { name: "Show section titles" });
    await user.click(checkbox);

    expect(settings.live.showSectionTitles).toBe(false);
  });

  it("closes modal when clicking Close button", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    settings.isModalOpen = true;

    render(SettingsModal);

    await user.click(screen.getByRole("button", { name: "Close" }));
    expect(settings.isModalOpen).toBe(false);
  });

  it("closes modal when clicking X button", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    settings.isModalOpen = true;

    render(SettingsModal);

    await user.click(screen.getByRole("button", { name: "Close settings" }));
    expect(settings.isModalOpen).toBe(false);
  });
});
