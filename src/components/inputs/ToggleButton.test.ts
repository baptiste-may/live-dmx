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
        label: "Red Cell",
      },
    });

    const button = screen.getByRole("button", { name: "Red Cell" });
    expect(button).toBeInTheDocument();
    expect(button).toHaveAttribute("aria-pressed", "true");
    expect(button).toHaveStyle({ backgroundColor: "rgb(255, 0, 0)" });
  });

  it("provides accessible name and aria-pressed in inactive cell mode", () => {
    render(ToggleButton, {
      props: {
        cell: true,
        color: "#00FF00",
        modelValue: false,
        label: "Green Cell",
      },
    });

    const button = screen.getByRole("button", { name: "Green Cell" });
    expect(button).toBeInTheDocument();
    expect(button).toHaveAttribute("aria-pressed", "false");
  });

  it("applies bg-warning and uses slot content as accessible name when color is not provided", () => {
    render(ToggleButton, {
      props: {
        cell: true,
        modelValue: false,
      },
      slots: {
        default: "Fallback Cell",
      },
    });

    const button = screen.getByRole("button", { name: "Fallback Cell" });
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass("bg-warning");
  });
});
