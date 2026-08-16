import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useShowChanges } from "@/stores/show/show-changes.ts";
import { useShowIdentity } from "@/stores/show/show-identity.ts";

import UnsavedChangesModal from "./UnsavedChangesModal.vue";

describe("UnsavedChangesModal.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should not render modals when isUnsavedChangesModalOpen is false", () => {
    const changesStore = useShowChanges();
    changesStore.isUnsavedChangesModalOpen = false;

    render(UnsavedChangesModal);

    expect(screen.queryByText("Unsaved Changes")).not.toBeInTheDocument();
  });

  it("should render modals with display name when isUnsavedChangesModalOpen is true", () => {
    const changesStore = useShowChanges();
    const identityStore = useShowIdentity();
    changesStore.isUnsavedChangesModalOpen = true;
    identityStore.displayName = "My Concert Show";

    render(UnsavedChangesModal);

    expect(screen.getByText("Unsaved Changes")).toBeInTheDocument();
    expect(screen.getByText("My Concert Show")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Discard Changes" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save & Continue" })).toBeInTheDocument();
  });

  it("should close modals when Cancel button is clicked", async () => {
    const user = userEvent.setup();
    const changesStore = useShowChanges();
    changesStore.isUnsavedChangesModalOpen = true;

    render(UnsavedChangesModal);

    await user.click(screen.getByRole("button", { name: "Cancel" }));

    expect(changesStore.isUnsavedChangesModalOpen).toBe(false);
  });

  it("should discard changes and execute pending action when Discard Changes button is clicked", async () => {
    const user = userEvent.setup();
    const changesStore = useShowChanges();
    const pendingAction = vi.fn();
    changesStore.isUnsavedChangesModalOpen = true;
    changesStore.pendingAction = pendingAction;

    render(UnsavedChangesModal);

    await user.click(screen.getByRole("button", { name: "Discard Changes" }));

    expect(changesStore.isUnsavedChangesModalOpen).toBe(false);
    expect(changesStore.isDirty).toBe(false);
    expect(pendingAction).toHaveBeenCalledTimes(1);
  });

  it("should save changes and execute pending action when Save & Continue button is clicked", async () => {
    const user = userEvent.setup();
    const changesStore = useShowChanges();
    const pendingAction = vi.fn();
    changesStore.isUnsavedChangesModalOpen = true;
    changesStore.pendingAction = pendingAction;
    changesStore.path = "/shows/test.ldmx";

    render(UnsavedChangesModal);

    await user.click(screen.getByRole("button", { name: "Save & Continue" }));

    expect(changesStore.save).toHaveBeenCalledTimes(1);
  });
});
