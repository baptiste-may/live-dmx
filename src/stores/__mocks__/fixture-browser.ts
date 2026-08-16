import type { FixtureType, Manufacturers } from "@/types/fixtures.ts";

const createMockFixture = (
  name: string,
  fixtureKey: string,
  manufacturerKey: string,
): FixtureType => ({
  name,
  fixtureKey,
  manufacturerKey,
  categories: ["Color Changer"],
  modes: [{ name: "Standard", channels: ["Dimmer", "Red", "Green", "Blue"] }],
});

export const mockFixturesData: Manufacturers = {
  chauvet: {
    name: "Chauvet DJ",
    fixtures: {
      "gig-bar-2": createMockFixture("GigBAR 2", "gig-bar-2", "chauvet"),
      "slimpar-64": createMockFixture("SlimPAR 64", "slimpar-64", "chauvet"),
    },
  },
  adj: {
    name: "American DJ",
    fixtures: {
      "mega-par-profile": createMockFixture("Mega Par Profile", "mega-par-profile", "adj"),
    },
  },
};
