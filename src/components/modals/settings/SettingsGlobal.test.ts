import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useSettings } from "@/stores/settings.ts";

import SettingsGlobal from "./SettingsGlobal.vue";

describe("SettingsGlobal.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders theme selector and scale slider", () => {
    render(SettingsGlobal);

    expect(screen.getByText("Global")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Dark theme" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Light theme" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "System theme" })).toBeInTheDocument();
    expect(screen.getByRole("slider", { name: "Scale" })).toBeInTheDocument();
  });

  it("changes theme when clicking theme buttons", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    render(SettingsGlobal);

    await user.click(screen.getByRole("button", { name: "Dark theme" }));
    expect(settings.theme).toBe("dark");

    await user.click(screen.getByRole("button", { name: "Light theme" }));
    expect(settings.theme).toBe("light");

    await user.click(screen.getByRole("button", { name: "System theme" }));
    expect(settings.theme).toBe("system");
  });

  it("updates scale when clicking ruler labels", async () => {
    const user = userEvent.setup();
    const settings = useSettings();
    render(SettingsGlobal);

    await user.click(screen.getByText("75%"));
    expect(settings.scale).toBe(75);
  });
});
