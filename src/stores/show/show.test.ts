import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { defaultShow, ShowStatus, useShow } from "./show";

describe("show store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with NOT_LOADED status", () => {
    const show = useShow();

    expect(show.state).toBe(ShowStatus.NOT_LOADED);
    expect(show.isLoaded).toBe(false);
    expect(show.isLoading).toBe(false);
    expect(show.isSaving).toBe(false);
    expect(show.hasUnsavedChanges).toBe(false);
    expect(show.canSave).toBe(false);
    expect(show.canUndo).toBe(false);
    expect(show.canRedo).toBe(false);
  });

  it("should load show data and transition to LOADED status", () => {
    const show = useShow();

    show.load({
      identity: {
        displayName: "Loaded Show",
        nameForm: { isEditing: false, nameInput: "" },
      },
      file: {
        path: "/shows/test.json",
        isDirty: false,
        isUnsavedChangesModalOpen: false,
        pendingAction: null,
        canUndo: true,
        canRedo: false,
      },
      patch: { fixtures: [] },
    });

    expect(show.state).toBe(ShowStatus.LOADED);
    expect(show.isLoaded).toBe(true);
    expect(show.identity.displayName).toBe("Loaded Show");
    expect(show.file.path).toBe("/shows/test.json");
    expect(show.hasUnsavedChanges).toBe(false);
    expect(show.canSave).toBe(false);
    expect(show.canUndo).toBe(true);
    expect(show.canRedo).toBe(false);
  });

  it("should allow saving a new show at any time even without changes", () => {
    const show = useShow();

    show.load({
      identity: {
        displayName: "Brand New Show",
        nameForm: { isEditing: false, nameInput: "" },
      },
      file: {
        path: undefined,
        isDirty: false,
        isUnsavedChangesModalOpen: false,
        pendingAction: null,
        canUndo: false,
        canRedo: false,
      },
      patch: { fixtures: [] },
    });

    expect(show.isLoaded).toBe(true);
    expect(show.hasUnsavedChanges).toBe(false);
    expect(show.canSave).toBe(true);
  });

  it("should report hasUnsavedChanges and canSave when show is dirty", () => {
    const show = useShow();

    show.load({
      identity: {
        displayName: "Existing Show",
        nameForm: { isEditing: false, nameInput: "" },
      },
      file: {
        path: "/shows/existing.ldmx",
        isDirty: true,
        isUnsavedChangesModalOpen: false,
        pendingAction: null,
        canUndo: false,
        canRedo: false,
      },
      patch: { fixtures: [] },
    });

    expect(show.hasUnsavedChanges).toBe(true);
    expect(show.canSave).toBe(true);
  });

  it("should reset all sub-stores and transition to NOT_LOADED when unloaded", () => {
    const show = useShow();

    show.load(defaultShow);
    expect(show.isLoaded).toBe(true);

    show.unload();
    expect(show.state).toBe(ShowStatus.NOT_LOADED);
    expect(show.isLoaded).toBe(false);
    expect(show.identity.displayName).toBe("My show");
  });
});
