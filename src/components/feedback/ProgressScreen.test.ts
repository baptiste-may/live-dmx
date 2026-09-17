import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import ProgressScreen from "@/components/feedback/ProgressScreen.vue";
import { useShow } from "@/stores/show/show";

describe("ProgressScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render the given title and the progress value", () => {
    const showStore = useShow();
    showStore.progress = 50;

    render(ProgressScreen, {
      props: {
        title: "test title",
      },
    });

    expect(screen.getByText("test title")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "50");
    expect(screen.getByText("50%")).toBeInTheDocument();
    expect(screen.getByRole("progressbar").firstElementChild!).toHaveStyle({ width: "50%" });
  });

  it("should update progress dynamically when store progress changes", async () => {
    render(ProgressScreen, {
      props: {
        title: "Loading fixtures...",
      },
    });

    const showStore = useShow();
    showStore.progress = 25;
    await nextTick();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "25");
    expect(screen.getByRole("progressbar").firstElementChild!).toHaveStyle({ width: "25%" });

    showStore.progress = 75;
    await nextTick();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "75");
    expect(screen.getByRole("progressbar").firstElementChild!).toHaveStyle({ width: "75%" });
  });

  it("should clamp out-of-range progress values", async () => {
    render(ProgressScreen, {
      props: {
        title: "Loading fixtures...",
      },
    });

    const showStore = useShow();
    showStore.progress = 140;
    await nextTick();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "100");
  });

  it("should display the backend status message when provided", async () => {
    render(ProgressScreen, {
      props: {
        title: "Importing MVR...",
      },
    });

    const showStore = useShow();
    showStore.statusMessage = "Parsing fixtures...";
    await nextTick();

    expect(screen.getByText("Parsing fixtures...")).toBeInTheDocument();
  });
});
