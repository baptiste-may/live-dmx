import { render, screen } from "@testing-library/vue";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import SavingBar from "@/components/feedback/SavingBar.vue";
import { ShowStatus, useShow } from "@/stores/show/show";

describe("SavingBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("should be hidden while no save is running", () => {
    render(SavingBar);

    expect(screen.queryByRole("progressbar")).not.toBeInTheDocument();
  });

  it("should appear with an initial progress while saving", async () => {
    const showStore = useShow();
    render(SavingBar);

    showStore.state = ShowStatus.SAVING;
    await nextTick();

    const bar = screen.getByRole("progressbar");
    expect(Number(bar.getAttribute("aria-valuenow"))).toBeGreaterThanOrEqual(8);
  });

  it("should fill up and stay visible for a minimum duration before hiding", async () => {
    const showStore = useShow();
    render(SavingBar);

    showStore.state = ShowStatus.SAVING;
    await nextTick();
    expect(screen.getByRole("progressbar")).toBeInTheDocument();

    showStore.state = ShowStatus.LOADED;
    await nextTick();

    vi.advanceTimersByTime(399);
    await nextTick();
    expect(screen.getByRole("progressbar")).toBeInTheDocument();

    vi.advanceTimersByTime(1);
    await nextTick();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "100");

    vi.advanceTimersByTime(200);
    await nextTick();
    expect(screen.queryByRole("progressbar")).not.toBeInTheDocument();
  });
});
