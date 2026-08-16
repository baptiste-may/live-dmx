import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ToastType, useToastStore } from "@/stores/toast.ts";

import ToastContainer from "./ToastContainer.vue";

describe("ToastContainer.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render active toasts from toast store", () => {
    const toastStore = useToastStore();
    toastStore.toasts = [
      { id: "1", type: ToastType.SUCCESS, message: "Show saved successfully" },
      { id: "2", type: ToastType.ERROR, message: "Failed to connect to Art-Net" },
    ];

    render(ToastContainer);

    expect(screen.getByText("Show saved successfully")).toBeInTheDocument();
    expect(screen.getByText("Failed to connect to Art-Net")).toBeInTheDocument();
    expect(screen.getAllByRole("alert")).toHaveLength(2);
  });

  it("should remove toast when close button is clicked", async () => {
    const user = userEvent.setup();
    const toastStore = useToastStore();
    toastStore.toasts = [{ id: "100", type: ToastType.INFO, message: "Notice message" }];

    render(ToastContainer);

    const closeButton = screen.getByRole("button", { name: "✕" });
    await user.click(closeButton);

    expect(toastStore.removeToast).toHaveBeenCalledWith("100");
  });
});
