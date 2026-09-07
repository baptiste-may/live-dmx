import { h } from "vue";

import NumberIcon from "./NumberIcon.vue";

export { default as FixtureIcon } from "./FixtureIcon.vue";
export { default as NumberIcon } from "./NumberIcon.vue";

export const createNumberIcon = (number: number) => () => h(NumberIcon, { number });
export const TwoIcon = createNumberIcon(2);
export const ThreeIcon = createNumberIcon(3);
export const FourIcon = createNumberIcon(4);
