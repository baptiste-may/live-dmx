import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { ActivityIcon, SunIcon } from "lucide-vue-next";
import { describe, expect, it } from "vitest";

import LiveEffectsRow from "./LiveEffectsRow.vue";

describe("LiveEffectsRow.vue", () => {
  const mockEffects = {
    pulse: { label: "Pulse", icon: ActivityIcon },
    full: { label: "Full", icon: SunIcon },
  };

  const renderRow = (props: Record<string, unknown> = {}) => {
    const table = document.createElement("table");
    const tbody = document.createElement("tbody");
    table.appendChild(tbody);
    document.body.appendChild(table);

    return render(LiveEffectsRow, {
      props: {
        groupName: "Test Group",
        effects: mockEffects,
        currentEffect: null,
        currentAlignment: null,
        isBounce: false,
        ...props,
      },
      container: tbody,
    });
  };

  it("renders group name, effect buttons, width knob, and action buttons", () => {
    renderRow();

    expect(screen.getByText("Test Group :")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Pulse" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Full" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Bounce" })).toBeInTheDocument();
    expect(screen.getByRole("slider", { name: "Width" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Reset" })).toBeInTheDocument();
  });

  it("emits select-effect when an effect button is clicked", async () => {
    const user = userEvent.setup();
    const { emitted } = renderRow({ currentEffect: null });

    const pulseButton = screen.getByRole("button", { name: "Pulse" });
    await user.click(pulseButton);
    expect(emitted()["select-effect"]).toEqual([["pulse"]]);
  });

  it("emits select-alignment when an alignment button is clicked", async () => {
    const user = userEvent.setup();
    const { emitted } = renderRow({ currentAlignment: null });

    const leftButton = screen.getByRole("button", { name: "Left" });
    await user.click(leftButton);
    expect(emitted()["select-alignment"]).toEqual([["left"]]);
  });

  it("emits toggle-bounce and reset events", async () => {
    const user = userEvent.setup();
    const { emitted } = renderRow({ isBounce: false });

    const bounceButton = screen.getByRole("button", { name: "Bounce" });
    await user.click(bounceButton);
    expect(emitted()["toggle-bounce"]).toEqual([[true]]);

    const resetButton = screen.getByRole("button", { name: "Reset" });
    await user.click(resetButton);
    expect(emitted()["reset"]).toBeTruthy();
  });

  it("emits set-width when knob value changes", async () => {
    const user = userEvent.setup();
    const { emitted } = renderRow({ width: 100 });

    const knob = screen.getByRole("slider", { name: "Width" });
    knob.focus();
    await user.keyboard("{ArrowDown}");

    expect(emitted()["set-width"]).toEqual([[99]]);
  });

  it("renders group header with title attribute and truncate classes for long names", () => {
    renderRow({ groupName: "A Very Long Fixture Group Name" });

    const header = screen.getByTitle("A Very Long Fixture Group Name");
    expect(header).toBeInTheDocument();
    expect(header).toHaveClass("truncate", "whitespace-nowrap");
  });
});
