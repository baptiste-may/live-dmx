import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import FlashButton from "./FlashButton.vue";

describe("FlashButton.vue", () => {
  it("renders default slot content", () => {
    render(FlashButton, {
      slots: {
        default: "Flash",
      },
    });

    expect(screen.getByRole("button", { name: "Flash" })).toBeInTheDocument();
  });

  it("updates modelValue on mousedown, mouseup, and mouseleave", async () => {
    const { emitted } = render(FlashButton, {
      props: {
        modelValue: false,
      },
      slots: {
        default: "Flash",
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
      },
      slots: {
        default: "Flash",
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
});
