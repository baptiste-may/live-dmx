import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import ProgressScreen from "@/components/feedback/ProgressScreen.vue";
import { useShow } from "@/stores/show/show";

describe("ProgressScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render with the correct title and progress", () => {
    render(ProgressScreen, {
      props: {
        title: "test title",
      },
    });

    const showStore = useShow();
    showStore.progress = 50;

    expect(screen.getByText("test title")).toBeInTheDocument();
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
    expect(screen.getByRole("progressbar")).toHaveValue(25);

    showStore.progress = 75;
    await nextTick();
    expect(screen.getByRole("progressbar")).toHaveValue(75);
  });
});
