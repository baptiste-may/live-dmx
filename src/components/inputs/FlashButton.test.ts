import { fireEvent, render, screen } from "@testing-library/vue";
import { SunIcon } from "lucide-vue-next";
import { describe, expect, it } from "vitest";

import FlashButton from "./FlashButton.vue";

describe("FlashButton.vue", () => {
  it("renders label when cell is false and applies square styling", () => {
    const { container } = render(FlashButton, {
      props: {
        label: "Flash Me",
      },
    });

    const button = screen.getByRole("button", { name: "Flash Me" });
    expect(button).toBeInTheDocument();
    expect(screen.getByText("Flash Me")).toBeInTheDocument();
    expect(container.querySelector("button")).toHaveClass("btn-square");
  });

  it("renders icon at the top and text at the bottom", () => {
    const { container } = render(FlashButton, {
      props: {
        icon: SunIcon,
        label: "Flash",
      },
    });

    const button = screen.getByRole("button", { name: "Flash" });
    expect(button).toBeInTheDocument();

    const svg = container.querySelector("svg");
    const label = screen.getByText("Flash");

    expect(svg).toBeInTheDocument();
    expect(label).toBeInTheDocument();
    // Icon is placed before the label in the DOM
    expect(svg?.compareDocumentPosition(label)).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
  });

  it("renders only icon when label is omitted", () => {
    const { container } = render(FlashButton, {
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
    const { container } = render(FlashButton, {
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
    const { container } = render(FlashButton, {
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
    const { container } = render(FlashButton, {
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

  it("updates modelValue on mousedown, mouseup, and mouseleave", async () => {
    const { emitted } = render(FlashButton, {
      props: {
        modelValue: false,
        label: "Flash",
      },
    });

    const button = screen.getByRole("button", { name: "Flash" });

    await fireEvent.mouseDown(button);
    expect(emitted("update:modelValue")).toContainEqual([true]);

    await fireEvent.mouseUp(button);
    expect(emitted("update:modelValue")).toContainEqual([false]);

    await fireEvent.mouseDown(button);
    await fireEvent.mouseLeave(button);
    expect(emitted("update:modelValue")).toContainEqual([false]);
  });

  it("updates modelValue on keyboard Space and Enter down/up, and clears on blur", async () => {
    const { emitted } = render(FlashButton, {
      props: {
        modelValue: false,
        label: "Flash",
      },
    });

    const button = screen.getByRole("button", { name: "Flash" });

    await fireEvent.keyDown(button, { key: " " });
    expect(emitted("update:modelValue")).toContainEqual([true]);

    await fireEvent.keyUp(button, { key: " " });
    expect(emitted("update:modelValue")).toContainEqual([false]);

    await fireEvent.keyDown(button, { key: "Enter" });
    expect(emitted("update:modelValue")).toContainEqual([true]);

    await fireEvent.blur(button);
    expect(emitted("update:modelValue")).toContainEqual([false]);
  });

  it("renders with cell styling and background color", () => {
    render(FlashButton, {
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
    render(FlashButton, {
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
    render(FlashButton, {
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

  it("renders check icon when active in cell mode", () => {
    const { container } = render(FlashButton, {
      props: {
        cell: true,
        modelValue: true,
        label: "Checked Cell",
      },
    });

    expect(container.querySelector("svg")).toBeInTheDocument();
  });
});
