import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useFixtureBrowser } from "@/stores/fixture-browser.ts";
import { usePatchForm } from "@/stores/patch/patch-form.ts";

import PatchFormBrowser from "./PatchFormBrowser.vue";

describe("PatchFormBrowser.vue", () => {
  it("should render search input and clear button", async () => {
    const user = userEvent.setup();
    render(PatchFormBrowser);

    const input = screen.getByPlaceholderText("Search fixtures...");
    expect(input).toBeInTheDocument();

    const browser = useFixtureBrowser();
    const patchForm = usePatchForm();

    const clearButton = screen.getByRole("button", { name: "Clear" });
    await user.click(clearButton);

    expect(browser.clearSearch).toHaveBeenCalled();
    expect(patchForm.$reset).toHaveBeenCalled();
  });
});
