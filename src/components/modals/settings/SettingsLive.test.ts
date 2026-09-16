import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useSettings } from "@/stores/settings.ts";

import SettingsLive from "./SettingsLive.vue";

describe("SettingsLive.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the live toggles", () => {
    render(SettingsLive);

    expect(screen.getByText("Live")).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Show button labels" })).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Show section titles" })).toBeInTheDocument();
  });

  it("toggles show button labels", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    settings.live.showButtonLabels = true;
    render(SettingsLive);

    await user.click(screen.getByRole("checkbox", { name: "Show button labels" }));

    expect(settings.live.showButtonLabels).toBe(false);
  });

  it("toggles show section titles", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    settings.live.showSectionTitles = true;
    render(SettingsLive);

    await user.click(screen.getByRole("checkbox", { name: "Show section titles" }));

    expect(settings.live.showSectionTitles).toBe(false);
  });
});
