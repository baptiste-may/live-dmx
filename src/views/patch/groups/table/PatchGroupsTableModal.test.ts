import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { usePatchGroups } from "@/stores/patch/patch-groups.ts";

import PatchGroupsTableModal from "./PatchGroupsTableModal.vue";

describe("PatchGroupsTableModal.vue", () => {
  it("should not render modal when modal.isOpen is false", () => {
    render(PatchGroupsTableModal);

    expect(screen.queryByText(/Edit/)).not.toBeInTheDocument();
  });

  it("should render modal and handle updates when modal.isOpen is true", async () => {
    const user = userEvent.setup();
    const patchGroups = usePatchGroups();
    patchGroups.table.selectedFixtures = [1, 2];
    patchGroups.table.modal.isOpen = true;
    patchGroups.table.modal.nameInput = "Updated Fixtures";

    render(PatchGroupsTableModal);

    expect(screen.getByText(/Edit 2 fixtures/)).toBeInTheDocument();

    const updateNameBtn = screen.getAllByRole("button", { name: "Update" })[0];
    await user.click(updateNameBtn);

    expect(patchGroups.updateSelectedFixturesName).toHaveBeenCalled();
  });
});
