import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { PageId, useNavigation } from "@/stores/navigation.ts";

import AppLayout from "./AppLayout.vue";

describe("AppLayout.vue", () => {
  it("should render AppSidebar and current navigation page", () => {
    render(AppLayout);

    const navStore = useNavigation();
    navStore.currentPage = PageId.PATCH;

    expect(screen.getByText("Show")).toBeInTheDocument();
    expect(screen.getByText("Setup")).toBeInTheDocument();
    expect(screen.getByText("Live")).toBeInTheDocument();
  });
});
