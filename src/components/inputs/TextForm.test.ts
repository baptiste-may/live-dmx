import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { ref } from "vue";

import TextForm from "./TextForm.vue";

describe("TextForm.vue", () => {
  it("renders text input and save/cancel buttons with initial modelValue", () => {
    render(TextForm, {
      props: {
        modelValue: "Initial Text",
        placeholder: "Enter text...",
      },
    });

    const input = screen.getByRole("textbox");
    expect(input).toHaveValue("Initial Text");
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
  });

  it("updates local state when typing and emits save/submit and update:modelValue when Save is clicked", async () => {
    const user = userEvent.setup();
    const { emitted } = render(TextForm, {
      props: {
        modelValue: "Initial",
      },
    });

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "Updated Text");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(emitted("update:modelValue")).toEqual([["Updated Text"]]);
    expect(emitted("save")).toEqual([["Updated Text"]]);
    expect(emitted("submit")).toEqual([["Updated Text"]]);
  });

  it("emits save/submit when Enter key is pressed inside input", async () => {
    const user = userEvent.setup();
    const { emitted } = render(TextForm, {
      props: {
        modelValue: "Group 1",
      },
    });

    const input = screen.getByRole("textbox");
    await user.type(input, " Modified{Enter}");

    expect(emitted("save")).toEqual([["Group 1 Modified"]]);
    expect(emitted("submit")).toEqual([["Group 1 Modified"]]);
  });

  it("emits cancel and resets value when Cancel button is clicked", async () => {
    const user = userEvent.setup();
    const { emitted } = render(TextForm, {
      props: {
        modelValue: "Original",
      },
    });

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "Draft Changes");
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    expect(emitted("cancel")).toHaveLength(1);
    expect(input).toHaveValue("Original");
    expect(emitted("save")).toBeUndefined();
  });

  it("emits cancel when Escape key is pressed", async () => {
    const user = userEvent.setup();
    const { emitted } = render(TextForm, {
      props: {
        modelValue: "Original",
      },
    });

    const input = screen.getByRole("textbox");
    await user.type(input, "{Escape}");

    expect(emitted("cancel")).toHaveLength(1);
  });

  it("disables input and buttons when disabled prop is true", () => {
    render(TextForm, {
      props: {
        modelValue: "Disabled Form",
        disabled: true,
      },
    });

    expect(screen.getByRole("textbox")).toBeDisabled();
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  });

  it("supports two-way binding with v-model", async () => {
    const user = userEvent.setup();
    const ParentComponent = {
      components: { TextForm },
      setup() {
        const text = ref("Test Value");
        return { text };
      },
      template:
        '<div><TextForm v-model="text" /><span data-testid="output">{{ text }}</span></div>',
    };

    render(ParentComponent);

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "New Value");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(screen.getByTestId("output")).toHaveTextContent("New Value");
  });
});
