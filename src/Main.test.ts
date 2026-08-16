import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import Main from "./Main.vue";

describe("Main.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render main application layout with WipAlert, ToastContainer, and App", () => {
    const { container } = render(Main);

    const mainDiv = container.querySelector("div");
    expect(mainDiv).toBeInTheDocument();
    expect(mainDiv).toHaveClass("relative", "h-screen", "w-screen", "overflow-hidden");

    expect(screen.getByText("Live DMX")).toBeInTheDocument();
  });
});
