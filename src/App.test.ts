import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App.vue";
import { ShowStatus, useShow } from "./stores/show/show";
import { useShowChanges } from "./stores/show/show-changes";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

describe("App.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render Welcome component when show is not loaded", () => {
    const showStore = useShow();
    showStore.state = ShowStatus.NOT_LOADED;

    render(App);

    expect(screen.getByText("Live DMX")).toBeInTheDocument();
    expect(screen.queryByText("Loading show...")).not.toBeInTheDocument();
    expect(screen.queryByText("Saving show...")).not.toBeInTheDocument();
  });

  it("should render Loading component when show is loading", () => {
    const showStore = useShow();
    showStore.state = ShowStatus.LOADING;

    render(App);

    expect(screen.getByText("Loading show...")).toBeInTheDocument();
    expect(screen.queryByText("Saving show...")).not.toBeInTheDocument();
  });

  it("should render Saving component when show is saving", () => {
    const showStore = useShow();
    showStore.state = ShowStatus.SAVING;

    render(App);

    expect(screen.getByText("Saving show...")).toBeInTheDocument();
    expect(screen.queryByText("Loading show...")).not.toBeInTheDocument();
  });

  it("should render AppLayout component when show is loaded", () => {
    const showStore = useShow();
    showStore.state = ShowStatus.LOADED;

    render(App);

    expect(screen.queryByText("Loading show...")).not.toBeInTheDocument();
    expect(screen.queryByText("Saving show...")).not.toBeInTheDocument();
    expect(screen.getByText("Show")).toBeInTheDocument();
  });

  it("should register global shortcuts and handle undo/redo/save actions", async () => {
    const showStore = useShow();
    const changesStore = useShowChanges();
    showStore.state = ShowStatus.LOADED;
    changesStore.isDirty = true;

    const { unmount } = render(App);

    expect(register).toHaveBeenCalledWith(
      [
        "CommandOrControl+Z",
        "CommandOrControl+Shift+Z",
        "CommandOrControl+Y",
        "CommandOrControl+S",
      ],
      expect.any(Function),
    );

    const handler = vi.mocked(register).mock.calls[0][1];

    // Undo
    handler({ shortcut: "CommandOrControl+Z", state: "Pressed", id: 1 });
    expect(changesStore.undo).toHaveBeenCalledTimes(1);

    // Redo (Shift+Z)
    handler({ shortcut: "CommandOrControl+Shift+Z", state: "Pressed", id: 2 });
    expect(changesStore.redo).toHaveBeenCalledTimes(1);

    // Redo (Y)
    handler({ shortcut: "CommandOrControl+Y", state: "Pressed", id: 3 });
    expect(changesStore.redo).toHaveBeenCalledTimes(2);

    // Save
    handler({ shortcut: "CommandOrControl+S", state: "Pressed", id: 4 });
    expect(changesStore.save).toHaveBeenCalledTimes(1);

    // Released event should not trigger actions
    handler({ shortcut: "CommandOrControl+Z", state: "Released", id: 5 });
    expect(changesStore.undo).toHaveBeenCalledTimes(1);

    // Unmount should unregister shortcuts
    unmount();
    expect(unregisterAll).toHaveBeenCalled();
  });

  it("should not trigger shortcuts when show is not loaded", async () => {
    const showStore = useShow();
    const changesStore = useShowChanges();
    showStore.state = ShowStatus.NOT_LOADED;

    render(App);

    const handler = vi.mocked(register).mock.calls[0][1];
    handler({ shortcut: "CommandOrControl+Z", state: "Pressed", id: 1 });
    expect(changesStore.undo).not.toHaveBeenCalled();
  });

  it("should attach the console logger on mount", async () => {
    const { attachConsole } = await import("@tauri-apps/plugin-log");
    const showStore = useShow();
    showStore.state = ShowStatus.NOT_LOADED;

    render(App);

    await vi.waitFor(() => {
      expect(attachConsole).toHaveBeenCalled();
    });
  });
});
