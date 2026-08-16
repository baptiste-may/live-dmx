import { defineStore } from "pinia";

export const useConfirmation = defineStore("confirmation", {
  state: (): {
    isModalOpen: boolean;
    reason: string;
    callback: undefined | (() => void);
  } => ({
    isModalOpen: false,
    reason: "",
    callback: undefined,
  }),
  actions: {
    ask(reason: string, callback?: () => void) {
      this.reason = reason;
      this.callback = callback;
      this.isModalOpen = true;
    },
  },
});
