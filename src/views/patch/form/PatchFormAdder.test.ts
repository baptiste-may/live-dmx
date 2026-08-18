import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { usePatchForm } from "@/stores/patch/patch-form";
import { mockFixtureType, mockMode } from "@/views/patch/__mocks__/fixtures";

import PatchFormAdder from "./PatchFormAdder.vue";

describe("PatchFormAdder.vue", () => {
  it("should disable patch button when no fixture is selected", () => {
    render(PatchFormAdder);

    const patchButton = screen.getByRole("button", { name: "Patch" });
    expect(patchButton).toBeDisabled();
  });

  it("should show selected fixture details and call patch on submit", async () => {
    const user = userEvent.setup();
    const formStore = usePatchForm();

    formStore.selectedFixture = mockFixtureType;
    formStore.selectedMode = mockMode;
    formStore.name = "Spot 500 1";

    render(PatchFormAdder);

    const patchButton = screen.getByRole("button", { name: "Patch" });
    expect(patchButton).not.toBeDisabled();

    await user.click(patchButton);
    expect(formStore.patch).toHaveBeenCalledTimes(1);
  });
});
