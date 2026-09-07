import userEvent from "@testing-library/user-event";
import { fireEvent, render, screen } from "@testing-library/vue";
import { SunIcon } from "lucide-vue-next";
import { describe, expect, it } from "vitest";

import ToggleButton from "./ToggleButton.vue";

describe("ToggleButton.vue", () => {
  it("renders label when cell is false and applies square styling", () => {
    const { container } = render(ToggleButton, {
      props: {
        label: "Toggle Me",
      },
    });

    const button = screen.getByRole("button", { name: "Toggle Me" });
    expect(button).toBeInTheDocument();
    expect(screen.getByText("Toggle Me")).toBeInTheDocument();
    expect(container.querySelector("button")).toHaveClass("btn-square");
  });

  it("renders icon at the top and text at the bottom", () => {
    const { container } = render(ToggleButton, {
      props: {
        icon: SunIcon,
        label: "Full",
      },
    });

    const button = screen.getByRole("button", { name: "Full" });
    expect(button).toBeInTheDocument();

    const svg = container.querySelector("svg");
    const label = screen.getByText("Full");

    expect(svg).toBeInTheDocument();
    expect(label).toBeInTheDocument();
    // Icon is placed before the label in the DOM
    expect(svg?.compareDocumentPosition(label)).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
  });

  it("renders only icon when label is omitted", () => {
    const { container } = render(ToggleButton, {
      props: {
        icon: SunIcon,
      },
      attrs: {
        "aria-label": "Sun Icon Button",
      },
    });

    const button = screen.getByRole("button", { name: "Sun Icon Button" });
    expect(button).toBeInTheDocument();
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("defaults to btn-lg and rounded-lg for non-cell buttons", () => {
    const { container } = render(ToggleButton, {
      props: {
        label: "Default Size Button",
      },
    });

    const button = container.querySelector("button");
    expect(button).toHaveClass("btn-lg");
    expect(button).toHaveClass("btn-square");
    expect(button).toHaveClass("rounded-lg");
  });

  it("applies specified size class", () => {
    const { container } = render(ToggleButton, {
      props: {
        label: "Small Button",
        size: "sm",
      },
    });

    const button = container.querySelector("button");
    expect(button).toHaveClass("btn-sm");
    expect(button).toHaveClass("btn-square");
  });

  it("defaults to btn-sm for cell mode", () => {
    const { container } = render(ToggleButton, {
      props: {
        cell: true,
        label: "Cell Button",
      },
    });

    const button = container.querySelector("button");
    expect(button).toHaveClass("btn-sm");
    expect(button).toHaveClass("btn-square");
    expect(button).toHaveClass("rounded-lg");
  });

  it("emits update:modelValue when clicked", async () => {
    const user = userEvent.setup();
    const { emitted } = render(ToggleButton, {
      props: {
        modelValue: false,
        label: "Click",
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

  it("applies bg-warning in cell mode when color is not provided", () => {
    render(ToggleButton, {
      props: {
        cell: true,
        modelValue: false,
        label: "Fallback Cell",
      },
    });

    const button = screen.getByRole("button", { name: "Fallback Cell" });
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass("bg-warning");
  });

  it("supports momentary mode with mousedown and mouseup", async () => {
    const { emitted } = render(ToggleButton, {
      props: {
        mode: "momentary",
        modelValue: false,
        label: "Flash",
      },
    });

    const button = screen.getByRole("button", { name: "Flash" });
    await fireEvent.mouseDown(button);
    expect(emitted("update:modelValue")).toContainEqual([true]);

    await fireEvent.mouseUp(button);
    expect(emitted("update:modelValue")).toContainEqual([false]);
  });
});
