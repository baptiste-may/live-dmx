import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { LivePageId, useLiveNavigation } from "@/stores/live/live-navigation";

import LiveContent from "./LiveContent.vue";

describe("LiveContent.vue", () => {
  it("renders the current active page component", async () => {
    const navStore = useLiveNavigation();
    navStore.currentPageId = LivePageId.INTENSITY;

    const { rerender } = render(LiveContent);

    expect(screen.getByRole("heading", { name: "Masters" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Strobes" })).toBeInTheDocument();

    navStore.navigate(LivePageId.COLORS);
    await rerender({});

    expect(screen.getByRole("heading", { name: "Colors" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Fade" })).toBeInTheDocument();
  });
});
