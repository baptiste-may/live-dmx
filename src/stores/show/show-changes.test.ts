import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useRecentShows } from "../recent-shows";
import { ToastType, useToastStore } from "../toast";
import { ShowStatus, useShow } from "./show";
import { useShowChanges } from "./show-changes";
import { useShowIdentity } from "./show-identity";

describe("show-changes store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("should initialize with default state", () => {
    const store = useShowChanges();

    expect(store.path).toBeUndefined();
    expect(store.isDirty).toBe(false);
    expect(store.isUnsavedChangesModalOpen).toBe(false);
    expect(store.pendingAction).toBeNull();
    expect(store.canUndo).toBe(false);
    expect(store.canRedo).toBe(false);
  });

  it("should call show_undo only when canUndo is true", async () => {
    const changesStore = useShowChanges();

    changesStore.canUndo = false;
    await changesStore.undo();
    expect(invoke).not.toHaveBeenCalledWith("show_undo");

    changesStore.canUndo = true;
    await changesStore.undo();
    expect(invoke).toHaveBeenCalledWith("show_undo");
  });

  it("should call show_redo only when canRedo is true", async () => {
    const changesStore = useShowChanges();

    changesStore.canRedo = false;
    await changesStore.redo();
    expect(invoke).not.toHaveBeenCalledWith("show_redo");

    changesStore.canRedo = true;
    await changesStore.redo();
    expect(invoke).toHaveBeenCalledWith("show_redo");
  });

  it("should save existing show directly and reset dirty state", async () => {
    const changesStore = useShowChanges();
    const identityStore = useShowIdentity();
    const toastStore = useToastStore();
    const recentShows = useRecentShows();

    changesStore.path = "/shows/my-concert.ldmx";
    changesStore.isDirty = true;
    identityStore.displayName = "My Concert";

    vi.mocked(invoke).mockResolvedValue([
      { displayName: "My Concert", path: "/shows/my-concert.ldmx", updatedAt: 12345 },
    ]);

    const result = await changesStore.save();

    expect(result).toBe(true);
    expect(changesStore.isDirty).toBe(false);
    expect(invoke).toHaveBeenCalledWith("show_save", { path: "/shows/my-concert.ldmx" });
    expect(toastStore.toasts.some((t) => t.type === ToastType.SUCCESS)).toBe(true);
    expect(recentShows.shows.some((s) => s.path === "/shows/my-concert.ldmx")).toBe(true);
  });

  it("should handle error when show_save fails", async () => {
    const changesStore = useShowChanges();
    const toastStore = useToastStore();

    changesStore.path = "/shows/my-concert.ldmx";
    changesStore.isDirty = true;

    vi.mocked(invoke).mockRejectedValueOnce(new Error("Disk write error"));

    const result = await changesStore.save();

    expect(result).toBe(false);
    expect(changesStore.isDirty).toBe(true);
    expect(toastStore.toasts.some((t) => t.type === ToastType.ERROR)).toBe(true);
  });

  it("should execute action immediately when show has no unsaved changes in guard", () => {
    const show = useShow();
    const changes = useShowChanges();
    show.state = ShowStatus.LOADED;
    changes.isDirty = false;

    const action = vi.fn();
    changes.confirmWithSaveGuard(action);

    expect(action).toHaveBeenCalledTimes(1);
    expect(changes.isUnsavedChangesModalOpen).toBe(false);
  });

  it("should open modal and store pending action when show has unsaved changes in guard", () => {
    const show = useShow();
    const changes = useShowChanges();
    show.state = ShowStatus.LOADED;
    changes.isDirty = true;

    const action = vi.fn();
    changes.confirmWithSaveGuard(action);

    expect(action).not.toHaveBeenCalled();
    expect(changes.isUnsavedChangesModalOpen).toBe(true);
    expect(changes.pendingAction).toBe(action);
  });

  it("should dismiss modal on cancel", () => {
    const changes = useShowChanges();
    changes.isUnsavedChangesModalOpen = true;
    changes.pendingAction = vi.fn();

    changes.handleCancel();

    expect(changes.isUnsavedChangesModalOpen).toBe(false);
    expect(changes.pendingAction).toBeNull();
  });

  it("should discard changes, reset dirty, and execute pending action", async () => {
    const changes = useShowChanges();
    changes.isDirty = true;
    changes.isUnsavedChangesModalOpen = true;
    const action = vi.fn();
    changes.pendingAction = action;

    await changes.discardAndExecute();

    expect(changes.isUnsavedChangesModalOpen).toBe(false);
    expect(changes.isDirty).toBe(false);
    expect(changes.pendingAction).toBeNull();
    expect(action).toHaveBeenCalledTimes(1);
  });

  it("should save changes and execute pending action on saveAndExecute", async () => {
    const changes = useShowChanges();
    changes.path = "/shows/tour.ldmx";
    changes.isDirty = true;
    changes.isUnsavedChangesModalOpen = true;
    const action = vi.fn();
    changes.pendingAction = action;

    await changes.saveAndExecute();

    expect(changes.isUnsavedChangesModalOpen).toBe(false);
    expect(changes.isDirty).toBe(false);
    expect(action).toHaveBeenCalledTimes(1);
  });
});
