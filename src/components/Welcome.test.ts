import { getCurrentWindow } from "@tauri-apps/api/window";
import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useApp } from "@/stores/app";

import Welcome from "./Welcome.vue";

describe("Welcome.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render welcome screen title and action buttons", () => {
    render(Welcome);

    expect(screen.getByText("Live DMX")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "New show" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Load show" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Exit" })).toBeInTheDocument();
  });

  it("should call createShow when New show button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    const appStore = useApp();
    await user.click(screen.getByRole("button", { name: "New show" }));

    expect(appStore.createShow).toHaveBeenCalledTimes(1);
  });

  it("should call askLoadShow when Load show button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    const appStore = useApp();
    await user.click(screen.getByRole("button", { name: "Load show" }));

    expect(appStore.askLoadShow).toHaveBeenCalledTimes(1);
  });

  it("should close window when Exit button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    await user.click(screen.getByRole("button", { name: "Exit" }));

    expect(getCurrentWindow().close).toHaveBeenCalledTimes(1);
  });
});
