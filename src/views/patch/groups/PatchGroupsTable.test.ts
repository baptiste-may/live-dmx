import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useShowPatch } from "@/stores/show/show-patch";
import { mockPatchedFixture } from "@/views/patch/__mocks__/fixtures";

import PatchGroupsTable from "./PatchGroupsTable.vue";

describe("PatchGroupsTable.vue", () => {
  it("should render table headers and rows for patched fixtures", () => {
    const showPatch = useShowPatch();
    showPatch.fixtures = [mockPatchedFixture];

    render(PatchGroupsTable);

    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Patch")).toBeInTheDocument();
    expect(screen.getByText("Groups")).toBeInTheDocument();
    expect(screen.getByText("Spot 1")).toBeInTheDocument();
  });
});
