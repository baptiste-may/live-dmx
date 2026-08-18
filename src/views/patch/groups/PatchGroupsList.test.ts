import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { usePatchGroups } from "@/stores/patch/patch-groups.ts";

import PatchGroupsList from "./PatchGroupsList.vue";

describe("PatchGroupsList.vue", () => {
  it("should render group list and allow adding a group", async () => {
    const user = userEvent.setup();
    render(PatchGroupsList);

    const groupsStore = usePatchGroups();
    groupsStore.create = async () => 2;

    const input = screen.getByPlaceholderText("New group name...");
    await user.type(input, "Front Lights");

    const addButton = screen.getByRole("button", { name: "Add Group" });
    await user.click(addButton);

    expect(screen.getByText(/Groups \(\d+\)/)).toBeInTheDocument();
  });
});
