import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { LivePageId, useLiveNavigation } from "@/stores/live/live-navigation";

import LiveNavigation from "./LiveNavigation.vue";

describe("LiveNavigation.vue", () => {
  it("renders navigation buttons for all live pages", () => {
    render(LiveNavigation);

    expect(screen.getByRole("button", { name: "Intensity" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Colors" })).toBeInTheDocument();
  });

  it("navigates to selected page on button click", async () => {
    const user = userEvent.setup();
    render(LiveNavigation);

    const navStore = useLiveNavigation();
    expect(navStore.currentPageId).toBe(LivePageId.INTENSITY);

    const colorsButton = screen.getByRole("button", { name: "Colors" });
    await user.click(colorsButton);

    expect(navStore.currentPageId).toBe(LivePageId.COLORS);
  });
});
