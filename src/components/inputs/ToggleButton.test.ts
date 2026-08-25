import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import ToggleButton from "./ToggleButton.vue";

describe("ToggleButton.vue", () => {
  it("renders default slot content when cell is false", () => {
    render(ToggleButton, {
      slots: {
        default: "Toggle Me",
      },
    });

    expect(screen.getByRole("button", { name: "Toggle Me" })).toBeInTheDocument();
  });

  it("emits update:modelValue when clicked", async () => {
    const user = userEvent.setup();
    const { emitted } = render(ToggleButton, {
      props: {
        modelValue: false,
      },
      slots: {
        default: "Click",
      },
    });

    const button = screen.getByRole("button", { name: "Click" });
    await user.click(button);

    expect(emitted("update:modelValue")).toEqual([[true]]);
  });

  it("renders with cell styling and background color", () => {
    render(ToggleButton, {
      props: {
        cell: true,
        color: "#FF0000",
        modelValue: true,
      },
    });

    const button = screen.getByRole("button");
    expect(button).toBeInTheDocument();
    expect(button).toHaveStyle({ backgroundColor: "rgb(255, 0, 0)" });
  });
});
