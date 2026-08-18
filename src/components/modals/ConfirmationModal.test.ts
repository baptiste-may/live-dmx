import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useConfirmation } from "@/stores/confirmation";

import ConfirmationModal from "./ConfirmationModal.vue";

describe("ConfirmationModal.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should not render modal when isModalOpen is false", () => {
    const confirmationStore = useConfirmation();
    confirmationStore.isModalOpen = false;

    render(ConfirmationModal);

    expect(screen.queryByText("Confirmation")).not.toBeInTheDocument();
  });

  it("should render modal with reason when isModalOpen is true", () => {
    const confirmationStore = useConfirmation();
    confirmationStore.isModalOpen = true;
    confirmationStore.reason = "Are you sure you want to delete this?";

    render(ConfirmationModal);

    expect(screen.getByText("Confirmation")).toBeInTheDocument();
    expect(screen.getByText("Are you sure you want to delete this?")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Cancel/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Confirm/i })).toBeInTheDocument();
  });

  it("should close modal when Cancel button is clicked", async () => {
    const user = userEvent.setup();
    const confirmationStore = useConfirmation();
    const callback = vi.fn();
    confirmationStore.isModalOpen = true;
    confirmationStore.reason = "Delete item?";
    confirmationStore.callback = callback;

    render(ConfirmationModal);

    await user.click(screen.getByRole("button", { name: /Cancel/i }));

    expect(confirmationStore.isModalOpen).toBe(false);
    expect(callback).not.toHaveBeenCalled();
  });

  it("should execute callback and close modal when Confirm button is clicked", async () => {
    const user = userEvent.setup();
    const confirmationStore = useConfirmation();
    const callback = vi.fn();
    confirmationStore.isModalOpen = true;
    confirmationStore.reason = "Delete item?";
    confirmationStore.callback = callback;

    render(ConfirmationModal);

    await user.click(screen.getByRole("button", { name: /Confirm/i }));

    expect(callback).toHaveBeenCalledTimes(1);
    expect(confirmationStore.isModalOpen).toBe(false);
  });
});
